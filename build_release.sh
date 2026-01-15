#!/bin/bash
set -e

echo "Building Frontend..."
cd frontend
npm install
npm run build
cd ..

echo "Building Backend..."
cd backend
# Ensure we build for release
cargo build --release
cd ..

echo "Creating Release..."
mkdir -p release/static

# Copy artifacts
cp backend/target/release/backend release/soselh-registration-backend
cp -r frontend/dist/* release/static/
cp backend/Config.toml.example release/Config.toml

echo "Configuring Release..."
# Create a start script
cat > release/start.sh << 'EOF'
#!/bin/bash
cd "$(dirname "$0")"
./soselh-registration-backend
EOF
chmod +x release/start.sh

# Compress
echo "Compressing..."
zip -r soselh-registration-release.zip release/
rm -rf release

echo "Done! Release created at soselh-registration-release.zip"
