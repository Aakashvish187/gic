use crate::starter_engine::models::{GeneratedFile, ProjectConfig, TemplateGenerator};

pub struct DockerStarter;
pub struct DockerComposeStarter;

impl TemplateGenerator for DockerStarter {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile> {
        let language = config
            .get_answer("language")
            .map(|s| s.as_str())
            .unwrap_or("Node");

        let content = match language {
            "Node" => {
                r#"FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm install
COPY . .
EXPOSE 3000
CMD ["npm", "start"]
"#
            }
            "Python" => {
                r#"FROM python:3.9-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 8000
CMD ["python", "app.py"]
"#
            }
            "Java" => {
                r#"FROM maven:3.8-openjdk-11 AS builder
WORKDIR /app
COPY . .
RUN mvn clean package -DskipTests

FROM openjdk:11-jre-slim
WORKDIR /app
COPY --from=builder /app/target/*.jar app.jar
EXPOSE 8080
ENTRYPOINT ["java", "-jar", "app.jar"]
"#
            }
            "Go" => {
                r#"FROM golang:1.19-alpine AS builder
WORKDIR /app
COPY . .
RUN go build -o main .

FROM alpine:latest
WORKDIR /app
COPY --from=builder /app/main .
EXPOSE 8080
CMD ["./main"]
"#
            }
            "Rust" => {
                r#"FROM rust:1.68 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
CMD ["myapp"]
"#
            }
            "PHP" => {
                r#"FROM php:8.1-apache
COPY src/ /var/www/html/
EXPOSE 80
"#
            }
            _ => {
                r#"FROM ubuntu:latest
CMD ["echo", "Hello World"]
"#
            }
        };

        vec![GeneratedFile {
            path: "Dockerfile".to_string(),
            content: content.to_string(),
        }]
    }
}

impl TemplateGenerator for DockerComposeStarter {
    fn generate(&self, config: &ProjectConfig) -> Vec<GeneratedFile> {
        let stack = config
            .get_answer("stack")
            .map(|s| s.as_str())
            .unwrap_or("Node + PostgreSQL");

        let content = match stack {
            "Node + PostgreSQL" => {
                r#"version: '3.8'
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://user:password@db:5432/mydb
    depends_on:
      - db
  db:
    image: postgres:14-alpine
    environment:
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
      - POSTGRES_DB=mydb
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
"#
            }
            "Redis" => {
                r#"version: '3.8'
services:
  redis:
    image: redis:alpine
    ports:
      - "6379:6379"
    volumes:
      - redisdata:/data

volumes:
  redisdata:
"#
            }
            "MongoDB" => {
                r#"version: '3.8'
services:
  mongodb:
    image: mongo:latest
    ports:
      - "27017:27017"
    environment:
      - MONGO_INITDB_ROOT_USERNAME=admin
      - MONGO_INITDB_ROOT_PASSWORD=password
    volumes:
      - mongodata:/data/db

volumes:
  mongodata:
"#
            }
            "WordPress" => {
                r#"version: '3.8'
services:
  wordpress:
    image: wordpress:latest
    ports:
      - "8080:80"
    environment:
      - WORDPRESS_DB_HOST=db
      - WORDPRESS_DB_USER=wordpress
      - WORDPRESS_DB_PASSWORD=wordpress
      - WORDPRESS_DB_NAME=wordpress
    depends_on:
      - db
  db:
    image: mysql:5.7
    environment:
      - MYSQL_DATABASE=wordpress
      - MYSQL_USER=wordpress
      - MYSQL_PASSWORD=wordpress
      - MYSQL_RANDOM_ROOT_PASSWORD=1
    volumes:
      - db_data:/var/lib/mysql

volumes:
  db_data:
"#
            }
            "ELK" => {
                r#"version: '3.8'
services:
  elasticsearch:
    image: docker.elastic.co/elasticsearch/elasticsearch:7.17.9
    environment:
      - discovery.type=single-node
    ports:
      - "9200:9200"
  kibana:
    image: docker.elastic.co/kibana/kibana:7.17.9
    ports:
      - "5601:5601"
    depends_on:
      - elasticsearch
"#
            }
            "Prometheus + Grafana" => {
                r#"version: '3.8'
services:
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    depends_on:
      - prometheus
"#
            }
            "Nginx" => {
                r#"version: '3.8'
services:
  web:
    image: nginx:alpine
    ports:
      - "80:80"
    volumes:
      - ./html:/usr/share/nginx/html
"#
            }
            _ => {
                r#"version: '3.8'
services:
  app:
    image: busybox
    command: echo "Hello World"
"#
            }
        };

        vec![GeneratedFile {
            path: "docker-compose.yml".to_string(),
            content: content.to_string(),
        }]
    }
}
