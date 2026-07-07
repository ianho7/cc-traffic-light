# 1. 删除配置文件目录
Remove-Item -Recurse -Force "$env:APPDATA\CcTrafficLight"

# 2. 删除本地安装的 hook 工具（如果有）
Remove-Item -Recurse -Force "$env:LOCALAPPDATA\CcTrafficLight" -ErrorAction SilentlyContinue

# 3. 删除临时日志（如果有）
Remove-Item "$env:TEMP\cc-traffic-light-runtime.log" -ErrorAction SilentlyContinue

# 4. 清除开机自启注册表
Remove-ItemProperty -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run" -Name "CcTrafficLight" -ErrorAction SilentlyContinue