# ==============================================================================
# 🗄️ AWS RDS POSTGRESQL MANAGED DATABASE & SECURITY GROUPS
# ==============================================================================

# 1. Database Subnet Group (Binds isolated private subnets for DB placement)
resource "aws_db_subnet_group" "db_subnet" {
  name        = "owasp-lab-db-subnet-group"
  description = "Isolated private subnets for managed PostgreSQL database"
  subnet_ids  = aws_subnet.private[*].id

  tags = {
    Name        = "owasp-lab-db-subnet-group"
    Environment = var.environment
  }
}

# 2. Database Security Group (Inbound restricted ONLY to Web App instances)
# (Blocks all other public or external private traffic)
resource "aws_security_group" "db_sg" {
  name        = "owasp-lab-db-sg"
  description = "Restricts PostgreSQL port 5432 access to Web Application instances only"
  vpc_id      = aws_vpc.main.id

  ingress {
    description     = "PostgreSQL from Web Application Security Group"
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.web_app_sg.id]
  }

  egress {
    description = "Limit DB outbound traffic for security"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "owasp-lab-db-sg"
    Environment = var.environment
  }
}

# 3. Managed Amazon RDS PostgreSQL Database Instance
resource "aws_db_instance" "postgres" {
  identifier        = "owasp-lab-db"
  allocated_storage = 20
  engine            = "postgres"
  engine_version    = "16.1"
  instance_class    = "db.t4g.micro" # Production-ready Graviton optimized instance

  # Credentials (highly secure, dynamically injected/retrieved)
  db_name  = "owasp_lab"
  username = "postgres"
  password = "SecureSuperuserPassword1!" # Root/Owner DB user for migrations

  # High Availability & Disaster Recovery (Checklist Items 1 & 2)
  multi_az            = true # Provisions a hot standby replica in another AZ
  publicly_accessible = false # Strictly isolated from the public internet

  # Storage Configuration & Automatic Daily Backups
  storage_type            = "gp3"
  backup_retention_period = 7    # Keep daily snapshots for 7 days
  backup_window           = "03:00-04:00" # Automated daily backups at low-traffic hours
  maintenance_window      = "Sun:04:30-Sun:05:30"
  skip_final_snapshot     = true

  # Networking & Security Groups (Checklist Item 3)
  db_subnet_group_name   = aws_db_subnet_group.db_subnet.name
  vpc_security_group_ids = [aws_security_group.db_sg.id]

  # Encryption-at-Rest
  storage_encrypted = true

  tags = {
    Name        = "owasp-lab-postgres"
    Environment = var.environment
  }
}

# ==============================================================================
# OUTPUTS
# ==============================================================================

output "db_endpoint" {
  value       = aws_db_instance.postgres.endpoint
  description = "The connection endpoint of the PostgreSQL RDS Database"
}
