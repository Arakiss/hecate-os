# HecateOS Migrations

This directory contains migration scripts that are automatically applied during system updates.

## How Migrations Work

1. Each migration is a bash script with a timestamp prefix: `TIMESTAMP_description.sh`
2. Migrations are applied in order by timestamp
3. Applied migrations are tracked in `/etc/hecate/migrations.applied`
4. Each migration runs only once

## Creating a New Migration

```bash
# Generate timestamp
ts=$(date +%s)

# Create migration file
touch migrations/${ts}_my_migration.sh
chmod +x migrations/${ts}_my_migration.sh
```

## Migration Template

```bash
#!/bin/bash
#
# HecateOS Migration: Description
# What this migration does
#

set -e

# Your migration code here
echo "Applying migration..."

# Example: Update a config file
# sed -i 's/old/new/' /etc/hecate/config

echo "Migration complete"
```

## Guidelines

- Migrations must be idempotent (safe to run multiple times)
- Always use `set -e` to stop on errors
- Log what you're doing with `echo`
- Don't require user interaction
- Test migrations before committing

## Running Migrations Manually

```bash
# List migrations
hecate migrate --list

# Dry run
hecate migrate --dry-run

# Apply pending migrations
sudo hecate migrate
```
