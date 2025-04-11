# BigCoin-CLI 使用指南

## 项目简介

BigCoin-CLI 是一个功能强大的命令行工具，专为与 BigCoin 区块链合约交互而设计。该工具支持多种操作，包括初始化账户、添加矿工、领取奖励、代币转账等。通过多线程处理，本工具可以高效管理大量钱包地址，是 BigCoin 生态系统中的实用工具。

## 安装指南

### 方法一：从源码编译（推荐）

1. 确保系统已安装 Rust 环境：
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 更新 Rust 到最新版本
rustup update
```

2. 克隆并编译项目：
```bash
# 克隆仓库
git clone https://github.com/mgcnb666/bigcoin-bot
cd bigcoin-bot

# 编译项目
cargo build --release

sudo cp ./target/release/bigcoin-cli /usr/local/bin/

# 可执行文件位于 ./target/release/bigcoin-cli
```

### 方法二：下载预编译版本

您也可以从项目发布页直接下载对应操作系统的预编译版本。

## 使用前准备

### 1. 创建私钥文件

创建一个文本文件（例如 `keys.txt`），每行一个私钥：

```
0x123456789abcdef123456789abcdef123456789abcdef123456789abcdef1234
9876543210fedcba9876543210fedcba9876543210fedcba9876543210fedcba
```

**安全提示：** 私钥文件极其敏感，请务必妥善保管并设置严格的文件权限。

### 2. 确认网络连接

默认情况下，工具连接到 `https://api.mainnet.abs.xyz` 节点。确保您的网络可以访问该地址。

## 命令详解

### 基本使用格式

```bash
bigcoin-cli [选项] --path <私钥文件路径> <命令>
```

### 全局选项

- `-m, --max-threads <数量>`: 设置最大并发线程数（默认：20）
- `-p, --path <文件路径>`: 指定包含私钥的文件路径（必需）
- `--rpc <RPC地址>`: 设置RPC节点地址（默认：https://api.mainnet.abs.xyz）
- `-h, --help`: 显示帮助信息

### 1. 初始化账户（Initialize）

此命令用于初始化账户，是其他操作的前提条件。

```bash
bigcoin-cli --path keys.txt initialize
# 或使用简写
bigcoin-cli --path keys.txt init
```

**功能说明**：
- 检查每个地址是否已初始化
- 查询初始化费用
- 确认账户余额充足
- 发送初始化交易



### 2. 添加启动矿工（Add-Starter）

为账户添加启动矿工，开始挖矿过程：

```bash
bigcoin-cli --path keys.txt add-starter
# 或使用简写
bigcoin-cli --path keys.txt start
```

**注意**：每个账户只能添加一个启动矿工。

### 3. 领取奖励（Claim）

领取账户累积的挖矿奖励：

```bash
bigcoin-cli --path keys.txt claim --min-claim-amount 0.01
# 或使用简写
bigcoin-cli --path keys.txt claim -m 0.01
```

**参数说明**：
- `--min-claim-amount, -m`: 设置最小领取数量，低于此数额将不会进行领取操作

### 4. 转账（Transfer）

将代币从私钥地址转移到指定目标地址：

```bash
bigcoin-cli --path keys.txt transfer -r 0x目标地址 -m 0.01
# 或使用简写
bigcoin-cli --path keys.txt send -r 0x目标地址 -m 0.01
```

**参数说明**：
- `-r, --receiver`: 接收地址（必填）
- `-m, --min-transfer-amount`: 最小转账金额（必填）
  
**注意**：当地址的余额大于设定的最小转账金额时，将会转出该地址的全部余额。

### 5. 打印奖励信息（Print）

显示所有私钥地址的累积奖励和余额信息：

```bash
bigcoin-cli --path keys.txt print
```

此命令非常有用，可用于监控账户状态、查看待领取奖励和余额。

## 自动化脚本

为了简化操作，您可以使用自动化脚本定期检查奖励并自动领取和转账。以下是一个示例脚本：

```bash
#!/bin/bash

# 配置参数
KEYS_PATH="/root/bigcoin-bot/keys.txt"
BIGCOIN_CLI="/root/bigcoin-bot/target/release/bigcoin-cli"
TARGET_ADDRESS="接收地址"
MIN_REWARDS="2"  # 最小待领取奖励阈值
MIN_CLAIM_AMOUNT="0.01"
MIN_TRANSFER_AMOUNT="0.01"
LOG_FILE="/root/bigcoin-auto.log"

# 检查bc是否安装，如果没有则安装
if ! command -v bc &> /dev/null; then
    echo "正在安装bc工具..." | tee -a $LOG_FILE
    apt-get update && apt-get install -y bc
    echo "bc工具安装完成" | tee -a $LOG_FILE
fi

# 创建日志文件
touch $LOG_FILE

echo "自动化脚本已启动，每60秒检查一次。记录日志到 $LOG_FILE" | tee -a $LOG_FILE
echo "目标地址: $TARGET_ADDRESS" | tee -a $LOG_FILE

while true; do
    # 记录当前时间
    current_time=$(date "+%Y-%m-%d %H:%M:%S")
    echo "[$current_time] 开始检查..." | tee -a $LOG_FILE
    
    # 获取奖励信息
    rewards_output=$($BIGCOIN_CLI --path $KEYS_PATH print)
    echo "[$current_time] 奖励检查输出:" | tee -a $LOG_FILE
    echo "$rewards_output" | tee -a $LOG_FILE
    
    # 提取总待领取奖励
    if echo "$rewards_output" | grep -q "Total pending rewards:"; then
        total_rewards=$(echo "$rewards_output" | grep "Total pending rewards:" | awk '{print $4}')
        echo "[$current_time] 提取到总待领取奖励: $total_rewards" | tee -a $LOG_FILE
        
        # 检查总奖励是否为数字
        if [[ $total_rewards =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
            # 使用bc进行浮点数比较
            if (( $(echo "$total_rewards > $MIN_REWARDS" | bc -l) )); then
                echo "[$current_time] 待领取奖励($total_rewards)大于阈值($MIN_REWARDS)，执行claim操作..." | tee -a $LOG_FILE
                
                # 执行claim操作
                claim_output=$($BIGCOIN_CLI --path $KEYS_PATH claim --min-claim-amount $MIN_CLAIM_AMOUNT 2>&1)
                echo "[$current_time] Claim结果:" | tee -a $LOG_FILE
                echo "$claim_output" | tee -a $LOG_FILE
                
                # 延迟几秒等待claim完成
                sleep 30
                
                echo "[$current_time] 执行transfer操作..." | tee -a $LOG_FILE
                
                # 执行transfer操作
                transfer_output=$($BIGCOIN_CLI --path $KEYS_PATH transfer -r $TARGET_ADDRESS -m $MIN_TRANSFER_AMOUNT 2>&1)
                echo "[$current_time] Transfer结果:" | tee -a $LOG_FILE
                echo "$transfer_output" | tee -a $LOG_FILE
                
                echo "[$current_time] 操作完成" | tee -a $LOG_FILE
            else
                echo "[$current_time] 待领取奖励($total_rewards)小于阈值($MIN_REWARDS)，不执行操作" | tee -a $LOG_FILE
            fi
        else
            echo "[$current_time] 错误: 提取的奖励'$total_rewards'不是有效数字" | tee -a $LOG_FILE
        fi
    else
        echo "[$current_time] 错误: 无法找到'Total pending rewards:'字段" | tee -a $LOG_FILE
    fi
    
    echo "[$current_time] 等待下一次检查..." | tee -a $LOG_FILE
    echo "-------------------------------------------" | tee -a $LOG_FILE
    
    # 等待60秒
    sleep 60
done
```

### 将脚本设置为系统服务

为确保脚本持续运行，您可以将其配置为系统服务：

```bash
# 创建服务文件
cat > /etc/systemd/system/bigcoin-auto.service << EOF
[Unit]
Description=BigCoin Auto Claim and Transfer Service
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/bigcoin-bot
ExecStart=/bin/bash /root/bigcoin-bot/auto_bigcoin.sh
Restart=always
RestartSec=10s

[Install]
WantedBy=multi-user.target
EOF

# 启用并启动服务
systemctl daemon-reload
systemctl enable bigcoin-auto.service
systemctl start bigcoin-auto.service
```

## 高级使用技巧

### 1. 批量处理

私钥文件可以包含多个地址，程序会使用多线程并行处理：

```bash
# 使用30个线程处理100个地址
bigcoin-cli --max-threads 30 --path many_keys.txt initialize
```

### 2. 定时任务

除了系统服务外，您还可以使用crontab设置定时任务：

```bash
# 编辑crontab
crontab -e

# 添加每6小时执行一次领取操作
0 */6 * * * /path/to/bigcoin-cli --path /path/to/keys.txt claim -m 0.01
```

### 3. 转出全部余额

要转出所有地址的全部余额，设置一个很小的最小转账金额：

```bash
bigcoin-cli --path keys.txt transfer -r 0x目标地址 -m 0.000001
```

## 常见问题与解决方案

### 1. 连接问题

**问题：** 
```
[错误] failed get chain_id
```

**解决方案：**
- 检查网络连接
- 尝试更换RPC节点：`--rpc https://其他节点地址`
- 确认链ID是否正确（应为2741）

### 2. 余额不足

**问题：**
```
[地址] balance is not enough: [余额], init price: [价格]
```

**解决方案：**
- 向相关地址转入足够的资金
- 确认使用的是正确的网络和账户

### 3. 交易失败

**问题：** 交易发送但未确认

**解决方案：**
- 等待网络确认
- 在浏览器中查看交易状态：https://abscan.org/

## 安全建议

1. **私钥安全**：
   - 不要在公共电脑上使用
   - 定期备份私钥文件
   - 使用强密码加密存储私钥
   - 操作完成后，安全清除内存和历史记录

2. **操作安全**：
   - 首次使用时，先用小额测试
   - 使用专门的虚拟机或隔离环境运行
   - 避免在同一文件中存储大量高价值账户的私钥

3. **网络安全**：
   - 使用可信的网络环境
   - 避免在公共Wi-Fi上操作
   - 考虑使用VPN保护连接

## 相关合约信息

BigCoin-CLI 与以下合约交互：
- 主合约：[0x09Ee83D8fA0f3F03f2aefad6a82353c1e5DE5705](https://abscan.org/address/0x09Ee83D8fA0f3F03f2aefad6a82353c1e5DE5705)
- 代币合约：[0xDf70075737E9F96B078ab4461EeE3e055E061223](https://abscan.org/address/0xDf70075737E9F96B078ab4461EeE3e055E061223)

您可以在区块链浏览器（如 https://abscan.org/）上查看合约详情。

---

## 贡献与反馈

欢迎提交问题报告和改进建议。如有疑问，请通过项目仓库提交issue。

---

*免责声明：使用本工具的风险由用户自行承担。请确保了解区块链操作的风险，并妥善保管您的私钥。*
