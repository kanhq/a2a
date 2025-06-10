// written by LLM provider: doubao model: doubao-deepseek-chat
async function main(config, params) {
    // 创建AutoHotKey脚本内容
    const ahkScript = `
        #IfWinActive
        ^j::
            ; 激活微信窗口
            WinActivate, ahk_exe WeChat.exe
            WinWaitActive, ahk_exe WeChat.exe
            ; 发送Ctrl+F快捷键
            Send, ^f
            Sleep, 500
            ; 输入Nick
            Send, Nick
            Sleep, 500
            ; 按下回车
            Send, {Enter}
        return
    `;

    // 使用ShellAction执行AutoHotKey脚本
    const action = {
        kind: "shell",
        command: "open",
        args: [ahkScript],
        argAsFile: "wechat_search.ahk"  // 将脚本保存为临时文件
    };

    // 执行脚本
    const result = await doAction(action);
    return result;
}
