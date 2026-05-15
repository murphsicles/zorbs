# Zorbs Registry — Seed targets
# Populate local/staging database to match production baseline.
#
# Usage:
#   make seed         — seed local database
#   make seed-vps     — seed VPS production database

DATABASE_URL ?= postgres://zorbs:zorbs_dev@localhost:5432/zorbs
SEEDS := seeds/001_stdlib_packages.sql

.PHONY: seed seed-vps

seed: $(SEEDS)
	@echo "==> Seeding $(DATABASE_URL)"
	@PGPASSWORD=zorbs_dev psql "$(DATABASE_URL)" -f $(SEEDS) 2>/dev/null || \
	 PGPASSWORD=zorbs_dev psql -h localhost -U zorbs -d zorbs -f $(SEEDS)
	@echo "==> Seed complete"

seed-vps:
	@echo "==> Seeding VPS database"
	@ssh zorbs "docker exec -i zorbs-db-1 psql -U zorbs -d zorbs" < seeds/001_stdlib_packages.sql
	@echo "==> VPS seed complete"
