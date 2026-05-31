# ==============================================================================
# 🔐 AWS ACM CERTIFICATE & AUTOMATED DNS VALIDATION (Route53)
# ==============================================================================

# 1. Look up the existing Route53 DNS Hosted Zone for the primary domain
data "aws_route53_zone" "primary" {
  name         = var.domain_name
  private_zone = false
}

# 2. Request a Public SSL/TLS Certificate from AWS Certificate Manager (ACM)
resource "aws_acm_certificate" "cert" {
  domain_name       = var.subdomain_name
  validation_method = "DNS"

  # Enforce best-practice encryption protocols & tags
  key_algorithm = "RSA_2048"

  tags = {
    Name        = "owasp-lab-tls-cert"
    Environment = var.environment
  }

  lifecycle {
    create_before_destroy = true
  }
}

# 3. Create the automated CNAME Validation Records in the Route53 Hosted Zone
# (ACM provides the required challenge name and value dynamically)
resource "aws_route53_record" "validation" {
  for_each = {
    for dvo in aws_acm_certificate.cert.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = data.aws_route53_zone.primary.zone_id
}

# 4. Wait for AWS ACM to verify the Route53 challenge and issue the certificate
resource "aws_acm_certificate_validation" "cert_validation" {
  certificate_arn         = aws_acm_certificate.cert.arn
  validation_record_fqdns = [for record in aws_route53_record.validation : record.fqdn]
}
