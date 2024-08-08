#!/bin/bash

# Start the Docker daemon
echo "Starting Docker daemon..."
sudo systemctl start docker

# Check if Docker is running
if ! sudo systemctl is-active --quiet docker; then
    echo "Docker daemon is not running. Exiting..."
    exit 1
fi

# Build the Docker image
echo "Building Docker image..."
sudo docker build -t url_shortener .

# Check if the image was built successfully
if [ $? -ne 0 ]; then
    echo "Failed to build Docker image. Exiting..."
    exit 1
fi

# Run the Docker container
echo "Running Docker container..."
sudo docker run -d -p 8080:8080 --name url_shortener_dev url_shortener

# Check if the container is running
if ! sudo docker ps | grep -q url_shortener_dev; then
    echo "Failed to run Docker container. Exiting..."
    exit 1
fi

echo "Docker container is running. Application is accessible at http://localhost:8080"

# Set up the development environment
echo "Setting up development environment..."
# Add any additional setup commands here

# Run the application in development mode
echo "Running application in development mode..."
sudo docker exec -it url_shortener_dev /bin/bash -c "cargo run"