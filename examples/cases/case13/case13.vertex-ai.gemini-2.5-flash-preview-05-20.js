// written by LLM provider: vertex-ai model: gemini-2.5-flash-preview-05-20
async function main(config, params) {
  // 定义 AutoHotkey 脚本内容
  // 当按下 Ctrl+J 时，激活微信窗口，然后发送 'Ctrl+F' 快捷键，输入 "Nick"，然后按下回车
  const ahkScript = `
    #Requires AutoHotkey v2.0

    ; 定义热键 Ctrl+J
    ^j::
    {
        ; 激活标题中包含 "微信" 的窗口
        WinActivate "微信"
        ; 等待窗口激活，最多等待 2 秒
        WinWaitActive "微信",, 2
        ; 如果窗口激活成功
        if WinActive("微信")
        {
            ; 发送 Ctrl+F 快捷键，通常用于打开搜索框
            Send "^f"
            ; 稍作等待，确保搜索框出现并准备好接收输入
            Sleep 100
            ; 输入 "Nick"
            Send "Nick"
            ; 按下回车键
            Send "{Enter}"
        }
        else
        {
            ; 如果微信窗口未激活，可以添加一些错误处理或通知
            ; 例如：ToolTip "微信窗口未找到或激活失败！", 100, 100
        }
    }
    `;

  // 使用 ShellAction 执行 AutoHotkey 脚本
  // 'open' 命令会使用系统默认程序打开文件，对于 .ahk 文件，会用 AutoHotkey 解释器执行
  // 'argsAsFile' 将 ahkScript 内容保存为临时文件，并将其路径作为参数传递给 'open' 命令
  return await doAction({
    kind: "shell",
    command: "open",
    argsAsFile: "wechat_automation.ahk", // 临时文件名称
    args: [ahkScript], // AutoHotkey 脚本内容
  });
}
