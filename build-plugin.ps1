$targetPath = Join-Path "target" "debug"
$lockPath = Join-Path $targetPath "plugin.lock"

# The game skips hot reloading while the lock exists, so it never picks up a plugin whose PDB
# has not landed yet.
New-Item -ItemType Directory -Force $targetPath | Out-Null
New-Item -ItemType File -Force $lockPath | Out-Null
try
{
    &cargo build --all-features --package handmade_hero_plugin
}
finally
{
    Remove-Item $lockPath -ErrorAction SilentlyContinue
}
exit $LASTEXITCODE
