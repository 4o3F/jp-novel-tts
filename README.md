# JP-NOVEL-TTS

帮坛友写的一个把TXT格式的日文小说转为音频格式可以直接在移动设备上听的程序

需要手动填充需要的运行库和字典，最终的结构如下
```
.
├── assets
│   └── open_jtalk_dic_utf_8-1.11   // 此处存放open jtalk字典文件
├── jp-novel-tts.exe
├── model                           // 此处存放VoiceVox的模型文件
│   ├── ...
│   ├── psv0.bin
│   └── sd0.bin
├── onnxruntime.dll
├── onnxruntime_providers_shared.dll
└── voicevox_core.dll
```

支持
- [x] 音色选择
- [x] 句间停顿
- [x] 标点符号停顿
- [x] 声速调节

## Notes
+ 注意为了保证win7可以运行，onnxruntime要使用microsoft编译的1.17.3版本的，1.18的会导致推理错误，voicevox编译的使用的动态链接库win7不行
+ 为了适配onnxruntime，win7下需要VxKex来辅助主程序入口