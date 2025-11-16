$pluginPath = "target/debug/handmade_hero_plugin.dll"
$backupPath = "target/debug/handmade_hero_plugin_old.dll"
if (Test-Path $pluginPath)
{
    Copy-Item $pluginPath $backupPath
}
&cargo build --package handmade_hero_plugin