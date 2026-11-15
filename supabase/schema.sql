-- Fynexo Cloud Extension Store schema + automatic Warning trigger + RLS
-- Paste and run in Supabase SQL Editor.
--
-- If you isolate apps using a custom Postgres schema (e.g. `fynexo` instead of `public`):
--   1. Replace `fynexo.` below with `your_schema.` (e.g. `fynexo.`).
--   2. Also add `your_schema` to Supabase Dashboard -> Project Settings -> Data API -> Exposed schemas.
--   3. Set `DATABASE_SCHEMA=your_schema` in `.env.local` (or in Fynexo's API Config UI).
--
-- create schema if not exists fynexo;
-- grant usage on schema fynexo to anon, authenticated;

create table if not exists fynexo.extensions (
    id uuid primary key default gen_random_uuid(),
    slug text unique not null,
    name text not null,
    category text not null default 'community',
    domain text not null default '*',
    description text not null default '',
    author text not null default 'community',
    version text not null default '1.0.0',
    css text not null default '',
    js text not null default '',
    size_bytes integer not null default 0,
    downloads integer not null default 0,
    -- 'unverified' | 'warning' | 'verified'
    status text not null default 'unverified',
    security_warning text not null default '',
    created_at timestamptz not null default now()
);

-- If upgrading an existing extensions table, ensure security_warning & size_bytes exist:
alter table fynexo.extensions add column if not exists security_warning text not null default '';
alter table fynexo.extensions add column if not exists size_bytes integer not null default 0;
alter table fynexo.extensions add column if not exists downloads integer not null default 0;

-- Server-side security audit trigger:
-- Runs inside Postgres even if someone bypasses Fynexo and calls Supabase REST API directly.
create or replace function fynexo.fnx_audit_extension_before_write()
returns trigger
language plpgsql
as $$
declare
    flags text[] := array[]::text[];
    js_src text := coalesce(new.js, '');
    css_src text := coalesce(new.css, '');
    dom text := btrim(coalesce(new.domain, '*'));
    jwt_role text := coalesce(current_setting('request.jwt.claim.role', true), '');
begin
    new.domain := case when dom = '' then '*' else dom end;
    new.size_bytes := octet_length(css_src) + octet_length(js_src);

    if length(btrim(js_src)) > 0 then
        if js_src ~* '\y(eval|Function)\s*\(|\y(setTimeout|setInterval)\s*\(\s*[''"`]' then
            flags := array_append(flags, 'Dynamic code execution (eval / Function)');
        end if;
        if js_src ~* '\y(fetch|XMLHttpRequest|sendBeacon|WebSocket|EventSource)\y' then
            flags := array_append(flags, 'Network request API (fetch / XHR / WS)');
        end if;
        if js_src ~* 'document\s*\.\s*cookie|\y(localStorage|sessionStorage|indexedDB)\y' then
            flags := array_append(flags, 'Cookie or web storage access');
        end if;
        if js_src ~* 'createElement\s*\(\s*[''"`](script|iframe)[''"`]|<(script|iframe)\y' then
            flags := array_append(flags, 'Dynamic <script> / <iframe> injection');
        end if;
        if js_src ~* '\y(atob|fromCharCode)\y|(\\x[0-9a-f]{2}){4,}|(\\u[0-9a-f]{4}){4,}' then
            flags := array_append(flags, 'Encoded / obfuscated code');
        end if;
        if js_src ~* 'type\s*=\s*[''"]?password|addEventListener\s*\(\s*[''"]submit[''"]' then
            flags := array_append(flags, 'Password / form submission interception');
        end if;
        if js_src ~* '\ywindow\s*\.\s*ipc\y|__fynexo_|__TAURI_' then
            flags := array_append(flags, 'Browser IPC / bridge probing');
        end if;
        if new.domain = '*' then
            flags := array_append(flags, 'Wildcard domain (*) combined with JS');
        end if;
        if exists (
            select 1
            from unnest(regexp_split_to_array(js_src, E'\r?\n')) as line
            where length(line) > 600
        ) then
            flags := array_append(flags, 'Long minified/single-line JS (>600 chars)');
        end if;
    end if;

    if length(btrim(css_src)) > 0 then
        if css_src ~* '@import\y|url\s*\(\s*[''"]?\s*(https?:|//)' then
            flags := array_append(flags, 'External URL or @import in CSS');
        end if;
        if css_src ~* 'expression\s*\(|-moz-binding\s*:' then
            flags := array_append(flags, 'Legacy CSS script expression');
        end if;
    end if;

    -- Refresh warning reasons automatically on insert or payload change
    if tg_op = 'INSERT' or new.css is distinct from old.css or new.js is distinct from old.js or new.domain is distinct from old.domain then
        new.security_warning := array_to_string(flags, '; ');
    end if;

    -- Anonymous callers can never self-approve ('verified'); risky uploads automatically become 'warning'.
    if jwt_role = 'anon' or tg_op = 'INSERT' then
        if jwt_role = 'anon' or new.status not in ('verified', 'unverified', 'warning') then
            new.status := case when array_length(flags, 1) > 0 then 'warning' else 'unverified' end;
        elsif new.status = 'unverified' and array_length(flags, 1) > 0 then
            new.status := 'warning';
        end if;
    end if;

    return new;
end;
$$;

drop trigger if exists trg_fnx_audit_extension on fynexo.extensions;
create trigger trg_fnx_audit_extension
before insert or update on fynexo.extensions
for each row execute function fynexo.fnx_audit_extension_before_write();

-- Strict Row Level Security:
alter table fynexo.extensions enable row level security;

drop policy if exists "Public read verified extensions" on fynexo.extensions;
create policy "Public read verified extensions"
on fynexo.extensions
for select
to anon, authenticated
using (status = 'verified');

drop policy if exists "Public submit unverified or warning extensions" on fynexo.extensions;
create policy "Public submit unverified or warning extensions"
on fynexo.extensions
for insert
to anon, authenticated
with check (status in ('unverified', 'warning'));

-- Required when using a custom schema (remember to replace `public` with your schema name):
grant usage on schema public to anon, authenticated;
grant select, insert on fynexo.extensions to anon, authenticated;
