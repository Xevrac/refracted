# Refracted website

Marketing site for Refracted, plus a small Discord-SSO admin to manage games and communities.

## Setup

```bash
cd frontend
composer install
copy .env.example .env
php artisan key:generate
type nul > database\database.sqlite
php artisan migrate --seed
php artisan storage:link
npm install
npm run build
php artisan serve
```

## Discord admin login

1. Create a Discord application and OAuth2 redirect: `{APP_URL}/gateway/callback`
2. Set in `.env`:
   - `DISCORD_CLIENT_ID`
   - `DISCORD_CLIENT_SECRET`
   - `DISCORD_REDIRECT_URI=${APP_URL}/gateway/callback`
   - `DISCORD_ADMIN_IDS=` your Discord user ID (comma-separated if needed)
3. Visit `/admin/login` and continue with Discord

## Local admin (dev only)

When `APP_ENV=local`, seed creates:

- Email: `admin@refracted.local`
- Password: `password`

Use the form on `/admin/login`, or Discord SSO with `DISCORD_ADMIN_IDS`.

```bash
php artisan db:seed
```


- Public page: `/`
- Admin: `/admin` (games + communities CRUD with image upload)
