# Build script placeholder for Windows

# Note: On Windows, you may need to install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

Write-Host "Building Spectral Workbench..." -ForegroundColor Green

# Check if Rust is installed
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Rust is not installed!" -ForegroundColor Red
    Write-Host "Please install Rust from: https://rustup.rs/" -ForegroundColor Yellow
    exit 1
}

# Check if Python is installed
if (!(Get-Command python -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Python is not installed!" -ForegroundColor Red
    exit 1
}

# Install maturin if not present
Write-Host "Installing/updating maturin..." -ForegroundColor Cyan
python -m pip install --upgrade maturin

# Build wheel
Write-Host "Building Rust extension..." -ForegroundColor Cyan
Set-Location "rust-core"
maturin build --release
$build_exit_code = $LASTEXITCODE
Set-Location ".."

if ($build_exit_code -ne 0) {
    Write-Host "💥 maturin failed" -ForegroundColor Red
    exit 1
}

# Find and install the wheel
Write-Host "Installing wheel..." -ForegroundColor Cyan
$wheel = Get-ChildItem -Path "target\wheels\*.whl" | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if ($wheel) {
    python -m pip install --force-reinstall $wheel.FullName
} else {
    Write-Host "Error: No wheel file found!" -ForegroundColor Red
    exit 1
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful!" -ForegroundColor Green
    Write-Host ""
    Write-Host "To run the application:" -ForegroundColor Yellow
    Write-Host "  python -m spectral_workbench.main" -ForegroundColor White
} else {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}
