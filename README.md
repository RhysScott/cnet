# cnet | 成都东软学院校园网快速认证 

`cnet` 是一个专为成都东软学院校园网快速认证而生的CLI工具。

现在普通用户**不需要自己安装 Rust**，可以直接去 GitHub Releases 下载已经打包好的版本，解压后就能使用。

它可以帮你：
- 保存一次学号、密码、套餐信息
- 以后直接一条命令连上校园网
- 一条命令下线
- 把 `cnet` 加到命令行环境里，方便长期使用

---

## 最简单的使用方法

如果你不是开发者，推荐你直接下载已经编译好的版本。

### 第一步：打开 Releases 页面
去这个项目的 **Releases** 页面，下载适合你电脑系统的压缩包。

常见文件名示例：
- Windows 64 位：`cnet-windows-x64-v0.1.1.zip`
- Linux 64 位：`cnet-linux-x64-v0.1.1.tar.gz`
- macOS Intel：`cnet-macos-intel-v0.1.1.tar.gz`
- macOS Apple Silicon：`cnet-macos-apple-silicon-v0.1.1.tar.gz`

如果你不确定自己电脑是什么架构，可以简单按下面理解：
- 大部分普通 Windows 电脑：下载 `cnet-windows-x64`
- 大部分普通 Intel / AMD Linux 电脑：下载 `cnet-linux-x64`
- 较老的 32 位 Linux：下载 `cnet-linux-x86`
- Apple M1 / M2 / M3 Mac：下载 `cnet-macos-apple-silicon`
- Intel 芯片 Mac：下载 `cnet-macos-intel`
- ARM Linux 设备：按设备情况选择 `cnet-linux-arm64` 或 `cnet-linux-armv7`

---

## 下载后怎么用

### Windows
1. 下载 `.zip` 文件
2. 解压到一个你容易找到的文件夹
3. 打开终端（PowerShell 或 CMD）
4. 进入解压后的目录
5. 首次运行：

```powershell
.\cnet.exe setup
```

### macOS / Linux
1. 下载 `.tar.gz` 文件
2. 解压到一个你容易找到的文件夹
3. 打开终端
4. 进入解压后的目录
5. 首次运行：

```bash
./cnet setup
```

---

## 首次配置

第一次使用建议先运行：

### Windows
```powershell
.\cnet.exe setup
```

### macOS / Linux
```bash
./cnet setup
```

运行后，它会一步一步问你：
- 学号
- 密码
- 套餐

说明：
- 密码输入是明文显示的，不会隐藏，避免你以为没有输入成功
- 如果之前已经保存过信息，可以直接回车保留旧值
- 套餐通常会尝试自动获取；如果失败，也可以手动输入

设置完成后，它还会问你：

> 是否把 cnet 添加到当前用户 PATH？

建议选“是”。

这样以后你就不需要每次都先进入解压目录，可以直接在终端里输入：

```bash
cnet
```

---

## 平时怎么用

### 1）连接校园网

```bash
cnet
```

如果本地已经保存过配置，它会直接帮你认证。
如果还没有配置，它会自动进入设置流程。

---

### 2）下线

```bash
cnet offline
```

它会自动读取本地配置并发起下线请求。

---

### 3）手动添加到 PATH

如果你在 `setup` 时没有选择添加到 PATH，也可以之后手动执行：

### Windows
```powershell
.\cnet.exe add-to-path
```

### macOS / Linux
```bash
./cnet add-to-path
```

执行后：
- Windows 一般需要关闭终端再重新打开
- macOS / Linux 一般需要重新打开终端

之后再输入 `cnet` 就会更方便。

---

## 如果提示“找不到 cnet”怎么办

这通常说明：
- 你还没有把 `cnet` 加到 PATH
- 或者你已经加了 PATH，但终端还没重新打开

你可以按这个顺序排查。

### 方法 1：先在解压目录里直接运行

#### Windows
```powershell
.\cnet.exe setup
```

#### macOS / Linux
```bash
./cnet setup
```

如果这样能运行，说明程序本身没问题，只是 PATH 还没配置好。

### 方法 2：手动添加到 PATH

#### Windows
```powershell
.\cnet.exe add-to-path
```

#### macOS / Linux
```bash
./cnet add-to-path
```

执行后关闭终端，再重新打开。

---

## 配置文件在哪里

`cnet` 会把你的配置保存在当前用户自己的配置目录里，不会写到项目源码目录中。

这意味着：
- 同一台电脑上的不同用户互不影响
- Windows、Linux、macOS 都会按各自系统习惯保存

如果配置文件损坏，程序会提示你重新生成。

---

## 常见问题

### 1. 我忘了套餐名怎么办？
先运行：

```bash
cnet setup
```

程序通常会尝试自动读取可用套餐。
如果没有成功，你也可以手动输入。

### 2. 我换了密码怎么办？
重新运行：

```bash
cnet setup
```

把密码改成新的即可。

### 3. 我不小心输错了信息怎么办？
还是运行：

```bash
cnet setup
```

重新保存一遍就可以覆盖旧配置。

### 4. Windows、Linux、macOS 都能用吗？
可以。这个项目会为以下平台自动打包发布：
- Windows x64
- Linux x86 / x64 / ARM64 / ARMv7
- macOS Intel
- macOS Apple Silicon

---

## 开发者或高级用户：从源码编译

如果你想自己编译源码，可以使用下面的方法。

### 1）先安装 Rust
安装完成后，在终端输入：

```bash
rustc --version
```

如果能看到版本号，就说明安装成功了。

### 2）编译 release 版本
在项目目录执行：

```bash
cargo build --release
```

编译成功后，可执行文件通常在：

### macOS / Linux
```bash
target/release/cnet
```

### Windows
```powershell
target\release\cnet.exe
```

然后你可以像下载版一样使用它。

---

## 最常用命令速查

### 首次设置
```bash
cnet setup
```

### 连接校园网
```bash
cnet
```

### 下线
```bash
cnet offline
```

### 添加到 PATH
```bash
cnet add-to-path
```

---

## 一句话总结

如果你只记一件事：

1. 去 Releases 下载适合你系统的版本
2. 先运行 `cnet setup`
3. 平时运行 `cnet`
4. 需要下线时运行 `cnet offline`

就够了。
