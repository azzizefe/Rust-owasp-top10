variable "aws_region" {
  type        = string
  description = "AWS region to deploy resources"
  default     = "eu-west-1"
}

variable "environment" {
  type        = string
  description = "Environment name (e.g., production, staging)"
  default     = "production"
}

variable "vpc_cidr" {
  type        = string
  description = "CIDR block for the VPC"
  default     = "10.0.0.0/16"
}

variable "public_subnet_cidrs" {
  type        = list(string)
  description = "CIDR blocks for the public subnets (ALB entry points)"
  default     = ["10.0.1.0/24", "10.0.2.0/24"]
}

variable "private_subnet_cidrs" {
  type        = list(string)
  description = "CIDR blocks for the private subnets (Web App and Nginx hosts)"
  default     = ["10.0.10.0/24", "10.0.11.0/24"]
}

variable "cloudflare_ipv4_cidrs" {
  type        = list(string)
  description = "Official list of Cloudflare IPv4 ranges for security group locking"
  default = [
    "173.245.48.0/20",
    "103.21.244.0/22",
    "103.22.200.0/22",
    "103.31.4.0/22",
    "141.101.64.0/18",
    "108.162.192.0/18",
    "190.93.240.0/20",
    "188.114.96.0/20",
    "197.234.240.0/22",
    "198.41.128.0/17",
    "162.158.0.0/15",
    "104.16.0.0/13",
    "104.24.0.0/14",
    "172.64.0.0/13",
    "131.0.72.0/22"
  ]
}

variable "cloudflare_ipv6_cidrs" {
  type        = list(string)
  description = "Official list of Cloudflare IPv6 ranges for security group locking"
  default = [
    "2400:cb00::/32",
    "2606:4700::/32",
    "2803:f800::/32",
    "2405:b500::/32",
    "2405:8100::/32",
    "2a06:98c0::/29",
    "2c0f:f248::/32"
  ]
}

variable "rate_limit_threshold" {
  type        = number
  description = "Rate limiting threshold (requests per 5 minutes per IP)"
  default     = 300
}

variable "domain_name" {
  type        = string
  description = "The primary domain name (e.g., yourdomain.com)"
  default     = "yourdomain.com"
}

variable "subdomain_name" {
  type        = string
  description = "The specific subdomain for the web application (e.g., lab.yourdomain.com)"
  default     = "lab.yourdomain.com"
}
