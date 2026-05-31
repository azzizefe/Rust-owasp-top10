# ==============================================================================
# 🔑 AWS IAM ROLE & INSTANCE PROFILE (Zero-Disk Secrets Management)
# ==============================================================================

# 1. Trust Policy: Allows EC2 instances to assume this IAM Role
data "aws_iam_policy_document" "ec2_trust_policy" {
  statement {
    actions = ["sts:AssumeRole"]
    effect  = "Allow"

    principals {
      type        = "Service"
      identifiers = ["ec2.amazonaws.com"]
    }
  }
}

# 2. IAM Role: The identity assigned to the EC2 instances
resource "aws_iam_role" "app_role" {
  name               = "owasp-lab-app-role"
  assume_role_policy = data.aws_iam_policy_document.ec2_trust_policy.json

  tags = {
    Name        = "owasp-lab-app-role"
    Environment = var.environment
  }
}

# 3. IAM Policy: Enforces Least Privilege, granting read-only access strictly to our AWS Secret
data "aws_iam_policy_document" "secrets_read_policy_doc" {
  statement {
    effect    = "Allow"
    actions   = ["secretsmanager:GetSecretValue"]
    resources = ["arn:aws:secretsmanager:*:*:secret:owasp-lab/production-*"]
  }
}

resource "aws_iam_policy" "secrets_read_policy" {
  name        = "owasp-lab-secrets-read-policy"
  description = "Allows read-only access to the production application secrets in Secrets Manager"
  policy      = data.aws_iam_policy_document.secrets_read_policy_doc.json
}

# 4. Attach the Policy to our App Role
resource "aws_iam_role_policy_attachment" "app_policy_attach" {
  role       = aws_iam_role.app_role.name
  policy_arn = aws_iam_policy.secrets_read_policy.arn
}

# 5. IAM Instance Profile: Mapped to EC2 server instances during provisioning
resource "aws_iam_instance_profile" "app_instance_profile" {
  name = "owasp-lab-app-instance-profile"
  role = aws_iam_role.app_role.name
}

# ==============================================================================
# OUTPUTS
# ==============================================================================

output "iam_instance_profile_arn" {
  value       = aws_iam_instance_profile.app_instance_profile.arn
  description = "The ARN of the IAM Instance Profile to attach to EC2 instances"
}
