# ==============================================================================
# 🚀 AWS ECS FARGATE COMPUTE & ZERO-DOWNTIME ROLLING DEPLOYMENT
# ==============================================================================

# 1. AWS ECS Cluster (Logical namespace for our services)
resource "aws_ecs_cluster" "app_cluster" {
  name = "owasp-lab-cluster"

  setting {
    name  = "containerInsights"
    value = "enabled"
  }

  tags = {
    Name        = "owasp-lab-ecs-cluster"
    Environment = var.environment
  }
}

# 2. CloudWatch Log Group for container telemetry
resource "aws_cloudwatch_log_group" "ecs_logs" {
  name              = "/ecs/owasp-lab-app"
  retention_in_days = 30

  tags = {
    Name        = "owasp-lab-ecs-logs"
    Environment = var.environment
  }
}

# 3. ECS Task Execution Role (Allows ECS Agent to pull images and write logs)
resource "aws_iam_role" "ecs_execution_role" {
  name = "owasp-lab-ecs-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action    = "sts:AssumeRole"
        Effect    = "Allow"
        Principal = { Service = "ecs-tasks.amazonaws.com" }
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "ecs_execution_attach" {
  role       = aws_iam_role.ecs_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

# 4. ECS Task Definition: Specifications for running our container
resource "aws_ecs_task_definition" "app_task" {
  family                   = "owasp-lab-app-task"
  network_mode             = "awsvpc" # Required for Fargate
  requires_compatibilities = ["FARGATE"]
  cpu                      = "256"    # 0.25 vCPU
  memory                   = "512"    # 512MB RAM
  execution_role_arn       = aws_iam_role.ecs_execution_role.arn
  task_role_arn            = aws_iam_role.app_role.arn # Reuses the secrets access IAM role from iam.tf!

  container_definitions = jsonencode([
    {
      name      = "app"
      image     = "azzizefe/rust-owasp-top10:latest"
      essential = true
      portMappings = [
        {
          containerPort = 8080
          hostPort      = 8080
        }
      ]
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = aws_cloudwatch_log_group.ecs_logs.name
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "app"
        }
      }
      environment = [
        { name = "SECRETS_PROVIDER", value = "aws" },
        { name = "AWS_REGION", value = var.aws_region },
        { name = "AWS_SECRET_NAME", value = "owasp-lab/production" },
        { name = "APP_MODE", value = "secure" }
      ]
    }
  ])
}

# 5. AWS ECS Fargate Service (Orchestrates containers and controls rolling updates)
resource "aws_ecs_service" "app_service" {
  name            = "owasp-lab-service"
  cluster         = aws_ecs_cluster.app_cluster.id
  task_definition = aws_ecs_task_definition.app_task.arn
  launch_type     = "FARGATE"
  desired_count   = 2 # Multi-container active-active setup spanning AZs

  # 🚀 ZERO-DOWNTIME ROLLING UPDATE CONFIGURATION (Checklist Item 2)
  # - Minimum healthy percent (100%): Ensures two containers are ALWAYS fully operational
  #   and healthy before terminating older container versions during deployment sweeps.
  # - Maximum percent (200%): Allows ECS to spin up 2 new parallel containers during deploy
  #   (up to 4 active tasks) to enable clean switchovers.
  deployment_minimum_healthy_percent = 100
  deployment_maximum_percent         = 200

  network_configuration {
    subnets          = aws_subnet.private[*].id
    security_groups  = [aws_security_group.web_app_sg.id]
    assign_public_ip = false # Securely isolated inside the private subnets
  }

  load_balancer {
    target_group_arn = aws_lb_target_group.app_tg.arn
    container_name   = "app"
    container_port   = 8080
  }

  depends_on = [aws_lb_listener.https]

  tags = {
    Name        = "owasp-lab-ecs-service"
    Environment = var.environment
  }
}
