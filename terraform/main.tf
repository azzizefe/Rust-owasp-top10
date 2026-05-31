terraform {
  required_version = ">= 1.5.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# ==============================================================================
# 1. NETWORKING (VPC & SUBNETS)
# ==============================================================================

resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name        = "owasp-lab-vpc"
    Environment = var.environment
  }
}

resource "aws_internet_gateway" "igw" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name        = "owasp-lab-igw"
    Environment = var.environment
  }
}

# Public Subnets (For ALB and/or Cloudflare ingress)
resource "aws_subnet" "public" {
  count                   = length(var.public_subnet_cidrs)
  vpc_id                  = aws_vpc.main.id
  cidr_block              = var.public_subnet_cidrs[count.index]
  availability_zone       = data.aws_availability_zones.available.names[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name        = "owasp-lab-public-subnet-${count.index + 1}"
    Environment = var.environment
  }
}

# Private Subnets (For Web App instances, isolated from public internet)
resource "aws_subnet" "private" {
  count             = length(var.private_subnet_cidrs)
  vpc_id            = aws_vpc.main.id
  cidr_block        = var.private_subnet_cidrs[count.index]
  availability_zone = data.aws_availability_zones.available.names[count.index]

  tags = {
    Name        = "owasp-lab-private-subnet-${count.index + 1}"
    Environment = var.environment
  }
}

data.aws_availability_zones.available {
  state = "available"
}

# Route Tables
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.igw.id
  }

  tags = {
    Name        = "owasp-lab-public-rt"
    Environment = var.environment
  }
}

resource "aws_route_table_association" "public" {
  count          = length(var.public_subnet_cidrs)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

# ==============================================================================
# 2. SECURITY GROUPS & ORIGIN SHIELDING (Direct-to-Origin Bypass Protection)
# ==============================================================================

# ALB Security Group: Inbound 80/443 is strictly limited to Cloudflare IP ranges.
# Public internet (0.0.0.0/0) is completely blocked.
resource "aws_security_group" "alb_sg" {
  name        = "owasp-lab-alb-sg"
  description = "Security group for ALB, strictly restricted to Cloudflare Edge IPs"
  vpc_id      = aws_vpc.main.id

  # Inbound HTTP (80) restricted to Cloudflare Edge IPs
  ingress {
    description = "HTTP from Cloudflare Edge"
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = var.cloudflare_ipv4_cidrs
  }

  # Inbound HTTPS (443) restricted to Cloudflare Edge IPs
  ingress {
    description = "HTTPS from Cloudflare Edge"
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = var.cloudflare_ipv4_cidrs
  }

  # Outbound rule to communicate with downstream Web Application
  egress {
    description = "Outbound to private app subnet"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "owasp-lab-alb-sg"
    Environment = var.environment
  }
}

# Web Server Security Group: Accepts traffic exclusively from the ALB
resource "aws_security_group" "web_app_sg" {
  name        = "owasp-lab-app-sg"
  description = "Security group for Web Application, restricted to ALB only"
  vpc_id      = aws_vpc.main.id

  ingress {
    description     = "HTTP from ALB Only"
    from_port       = 80
    to_port         = 80
    protocol        = "tcp"
    security_groups = [aws_security_group.alb_sg.id]
  }

  ingress {
    description     = "HTTPS from ALB Only"
    from_port       = 443
    to_port         = 443
    protocol        = "tcp"
    security_groups = [aws_security_group.alb_sg.id]
  }

  egress {
    description = "Outbound to internet (e.g. for package updates or external secrets/Vault)"
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = {
    Name        = "owasp-lab-app-sg"
    Environment = var.environment
  }
}

# ==============================================================================
# 3. APPLICATION LOAD BALANCER (ALB)
# ==============================================================================

resource "aws_lb" "alb" {
  name               = "owasp-lab-alb"
  internal           = false
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb_sg.id]
  subnets            = aws_subnet.public[*].id

  enable_deletion_protection = false

  tags = {
    Name        = "owasp-lab-alb"
    Environment = var.environment
  }
}

resource "aws_lb_target_group" "app_tg" {
  name     = "owasp-lab-tg"
  port     = 80
  protocol = "HTTP"
  vpc_id   = aws_vpc.main.id

  health_check {
    enabled             = true
    path                = "/health"
    protocol            = "HTTP"
    port                = "80"
    interval            = 30
    timeout             = 5
    healthy_threshold   = 3
    unhealthy_threshold = 3
    matcher             = "200"
  }

  tags = {
    Name        = "owasp-lab-tg"
    Environment = var.environment
  }
}

resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.alb.arn
  port              = "80"
  protocol          = "HTTP"

  default_action {
    type = "redirect"

    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.alb.arn
  port              = "443"
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS13-1-2-2021-06" # Secure TLS profile enforcing high ciphers
  certificate_arn   = aws_acm_certificate_validation.cert_validation.certificate_arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.app_tg.arn
  }
}

# ==============================================================================
# 4. AWS WAFV2 INTEGRATION (OWASP CRS, Bot Control & Rate Limiting)
# ==============================================================================

resource "aws_wafv2_web_acl" "waf" {
  name        = "owasp-lab-waf-acl"
  description = "AWS WAF Web ACL protecting OWASP Top 10 Lab with Core Rule Sets"
  scope       = "REGIONAL"

  default_action {
    allow {}
  }

  # 1. OWASP Common Rule Set (CRS) - Blocks SQLi, XSS, LFI/RFI, and other top vulnerabilities
  rule {
    name     = "AWS-AWSManagedRulesCommonRuleSet"
    priority = 10

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesCommonRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedRulesCommonRuleSetMetric"
      sampled_requests_enabled   = true
    }
  }

  # 2. SQL Injection Rule Set - Heavy verification of queries and request bodies
  rule {
    name     = "AWS-AWSManagedRulesSQLiRuleSet"
    priority = 20

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesSQLiRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedRulesSQLiRuleSetMetric"
      sampled_requests_enabled   = true
    }
  }

  # 3. Known Bad Inputs Rule Set - Blocks known automated scanner tools and bots
  rule {
    name     = "AWS-AWSManagedRulesKnownBadInputsRuleSet"
    priority = 30

    override_action {
      none {}
    }

    statement {
      managed_rule_group_statement {
        name        = "AWSManagedRulesKnownBadInputsRuleSet"
        vendor_name = "AWS"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "AWSManagedRulesKnownBadInputsRuleSetMetric"
      sampled_requests_enabled   = true
    }
  }

  # 4. Custom Rate-Limiting Rule (Protects against brute-force and DDoS)
  rule {
    name     = "CustomRateLimitRule"
    priority = 40

    action {
      block {}
    }

    statement {
      rate_based_statement {
        limit              = var.rate_limit_threshold
        aggregate_key_type = "IP"
      }
    }

    visibility_config {
      cloudwatch_metrics_enabled = true
      metric_name                = "CustomRateLimitMetric"
      sampled_requests_enabled   = true
    }
  }

  visibility_config {
    cloudwatch_metrics_enabled = true
    metric_name                = "OwaspLabWafAclMetric"
    sampled_requests_enabled   = true
  }

  tags = {
    Name        = "owasp-lab-waf-acl"
    Environment = var.environment
  }
}

# Associate the WAF Web ACL with our Application Load Balancer
resource "aws_wafv2_web_acl_association" "alb_association" {
  resource_arn = aws_lb.alb.arn
  web_acl_arn  = aws_wafv2_web_acl.waf.arn
}

# ==============================================================================
# 5. OUTPUTS
# ==============================================================================

output "alb_dns_name" {
  value       = aws_lb.alb.dns_name
  description = "The public DNS name of the Load Balancer"
}

output "vpc_id" {
  value       = aws_vpc.main.id
  description = "The ID of the VPC"
}
