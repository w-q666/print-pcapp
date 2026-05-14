# Java Print WebSocket 接口文档

## 概述

Java Print 提供 WebSocket 长连接服务，用于在打印过程中**实时推送打印状态**给客户端。WebSocket 通道与 HTTP 打印请求通过 `sessionId` 关联，实现打印状态的异步通知。

- **协议**: `ws://` 或 `wss://`
- **端点路径**: `/print`
- **完整地址**: `ws://{host}:{port}/print`
- **默认端口**: `2024`
- **实现方式**: JSR 356 (Java WebSocket API)，基于 `@ServerEndpoint` 注解

---

## 连接生命周期

### 1. 建立连接

客户端发起 WebSocket 握手连接到 `ws://{host}:{port}/print`。

**连接成功后**，服务端立即返回一条 JSON 消息，包含当前会话的唯一标识 `sessionId`。

**返回消息格式**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": "d7b8c9a1-2e3f-4a5b-8c6d-9e0f1a2b3c4d"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | `Integer` | `0` 表示连接成功 |
| `msg` | `String` | `"操作成功"` |
| `data` | `String` | **当前 WebSocket 会话的唯一标识（sessionId）**，用于后续 HTTP 打印请求关联 |

**重要**: 客户端必须保存此 `sessionId`，后续调用 HTTP `/print/single` 接口时需作为 `sessionId` 参数传入，才能接收到该打印任务的实时状态推送。

---

### 2. 消息接收（客户端 → 服务端）

服务端可接收客户端发送的文本消息，但**当前版本未实现消息处理逻辑**，`@OnMessage` 方法体内代码已被注释。

```java
@OnMessage
public void onMessage(String message) {
    // 当前为空实现
}
```

> 客户端无需发送消息给服务端，WebSocket 通道在当前版本中是**纯单向推送（服务端→客户端）**。

---

### 3. 连接关闭

当客户端主动关闭连接或网络断开时：

1. 服务端从会话集合中移除该 `Session`
2. 在线连接计数器自减

关闭后，以该 `sessionId` 发起的打印任务将**无法收到后续状态推送**（消息静默失败，不报错）。

---

### 4. 错误处理

当 WebSocket 连接发生错误时，服务端输出错误堆栈到控制台（`printStackTrace()`），**不会**向客户端发送错误消息。

---

## 服务端推送消息

### 消息发送机制

| 方法 | 说明 |
|------|------|
| `SendMessage(Session, String)` | 向指定 WebSocket 会话发送文本消息 |
| `SendMessageById(String, String)` | 根据 `sessionId` 查找对应会话并发送消息 |
| `SendMessageAll(String)` | 向所有已连接的会话广播消息 |

打印状态推送使用 `SendMessageById` 方法，通过 `sessionId` 精确投递到对应客户端。

---

## 打印状态推送协议

### 数据格式

所有打印状态推送消息均为 **JSON 字符串**，格式与 HTTP 响应格式一致（`CommonResult` 序列化）：

```json
{
  "code": <Integer>,
  "msg": "<String>"
}
```

注意：打印状态推送消息的 `data` 字段**始终为 `null`**，没有实际数据负载。所有状态信息由 `code` 和 `msg` 字段承载。

### 打印生命周期状态码

打印过程的状态按以下顺序推送，每个环节对应一个状态码：

| 序号 | 状态码 | `msg` 值 | 触发时机 | 说明 |
|------|--------|----------|----------|------|
| 1 | `200000` | `开始准备` | 打印任务提交至线程池后 | 表示系统已接收打印请求，正在准备打印资源 |
| 2 | `200001` | `开始打印` | 调用 `job.print()` 后 | 打印作业已提交至打印机，开始实际打印 |
| 3 | `200004` | `数据传输完成` | `printDataTransferCompleted` 事件 | 仅 PDF 打印，数据已完全传输至打印机 |
| 4 | `200002` | `打印结束` | `printJobCompleted` 事件 | 仅 PDF 打印，打印作业正常完成 |

### 异常与警告状态码

以下状态码可能在打印过程中随时出现：

| 状态码 | `msg` 值 | 触发时机 | 说明 |
|--------|----------|----------|------|
| `200003` | `打印异常` | 打印过程中抛出异常 | 打印作业因异常中断，如打印机不可用、格式不支持等 |
| `200005` | `需要人工干预` | `printJobRequiresAttention` 事件 | 仅 PDF 打印，打印机需要人工操作（如卡纸、缺纸等） |
| `200006` | `打印失败` | `printJobFailed` 事件 | 仅 PDF 打印，打印作业失败 |
| `200007` | `打印取消` | `printJobCanceled` 事件 | 仅 PDF 打印，打印作业被取消 |

### 文件异常状态码

在获取打印文件阶段可能出现的错误：

| 状态码 | `msg` 值 | 说明 |
|--------|----------|------|
| `200008` | `系统找不到指定的文件` | `source=path` 时指定的文件不存在 |
| `200009` | `文件异常` | 文件读取或格式错误 |

---

## 推送消息示例

### 完整打印流程（PDF 类型）

以下是一次 PDF 打印作业的完整推送消息序列示例：

**第 1 条 — 开始准备**:
```json
{"code":200000,"msg":"开始准备"}
```

**第 2 条 — 开始打印**:
```json
{"code":200001,"msg":"开始打印"}
```

**第 3 条 — 数据传输完成**:
```json
{"code":200004,"msg":"数据传输完成"}
```

**第 4 条 — 打印结束**:
```json
{"code":200002,"msg":"打印结束"}
```

### 异常流程示例

**打印机不可用**:
```json
{"code":200000,"msg":"开始准备"}
```
```json
{"code":200003,"msg":"打印异常"}
```

**需要人工干预（卡纸等）**:
```json
{"code":200000,"msg":"开始准备"}
```
```json
{"code":200005,"msg":"需要人工干预"}
```

---

## 不同打印类型的状态推送差异

### PDF 打印（`type=PDF`）

支持**完整的打印生命周期监听**，会推送所有状态码：

- `200000` 开始准备
- `200001` 开始打印
- `200004` 数据传输完成
- `200002` 打印结束
- `200003` 打印异常
- `200005` 需要人工干预
- `200006` 打印失败
- `200007` 打印取消

推送机制：通过 `PrintJobListener` 注册到 `DocPrintJob`，监听 Java 打印服务 API 的所有打印事件。

### 图片打印（`type=IMG`）

仅推送以下状态码：

- `200000` 开始准备
- `200001` 开始打印
- `200003` 打印异常

**不推送** `200002`（打印结束）、`200004`~`200007` 等详细状态，因为未注册 `PrintJobListener`。

### 文本/HTML 打印（`type=TEXT` / `type=HTML`）

**当前不推送任何打印状态**。这两个打印服务类仅输出控制台日志。

---

## 会话管理

| 属性 | 说明 |
|------|------|
| 会话存储 | `CopyOnWriteArraySet<Session>` — 线程安全的集合 |
| 在线计数 | `AtomicInteger` — 原子计数器（当前接入/断开均有调用，但接入端被注释） |
| 最大并发 | 无硬性限制，受系统资源约束 |

### 会话查找

`SendMessageById` 通过遍历 `SessionSet`，比对 `session.getId()` 来查找目标会话，时间复杂度 O(n)。

**注意**: 每次浏览器刷新或 WebSocket 重连，`sessionId` 都会变化。如果打印任务依赖旧的 `sessionId`，状态推送将无法到达。

---

## 与 HTTP 接口的协作流程

```
┌──────────┐                    ┌──────────┐                    ┌──────────┐
│  客户端   │                    │  HTTP API │                    │ WebSocket│
│ (浏览器)  │                    │  /print   │                    │ /print   │
└────┬─────┘                    └────┬─────┘                    └────┬─────┘
     │                               │                               │
     │  1. WebSocket 连接            │                               │
     │──────────────────────────────────────────────────────────────>│
     │                               │                               │
     │  2. 返回 sessionId             │                               │
     │<──────────────────────────────────────────────────────────────│
     │                               │                               │
     │  3. POST /print/single        │                               │
     │  (携带 sessionId)              │                               │
     │──────────────────────────────>│                               │
     │                               │                               │
     │  4. 返回 HTTP 200              │                               │
     │<──────────────────────────────│                               │
     │                               │                               │
     │                               │  5. SendMessageById            │
     │                               │     (sessionId, 打印状态)      │
     │                               │──────────────────────────────>│
     │                               │                               │
     │  6. 推送 {"code":200000,...}   │                               │
     │<──────────────────────────────────────────────────────────────│
     │                               │                               │
     │  7. 推送 {"code":200001,...}   │                               │
     │<──────────────────────────────────────────────────────────────│
     │                               │                               │
     │  8. 推送 {"code":200002,...}   │                               │
     │<──────────────────────────────────────────────────────────────│
```

### 步骤说明

1. **建立 WebSocket 连接**: 客户端在页面初始化时连接到 `ws://{host}:{port}/print`
2. **获取 sessionId**: 服务端在 `@OnOpen` 时立即返回 `CommonResult` 格式的 sessionId
3. **发起打印请求**: 调用 HTTP `POST /print/single`，将 `sessionId` 作为请求参数传入
4. **HTTP 响应**: 接口立即返回 `{"code":0,"msg":"操作成功"}`，此时打印已提交至后台线程池
5. **状态推送**: 打印服务通过 `SendMessageById(sessionId, status)` 推送状态
6-8. **客户端接收**: 客户端实时收到各阶段打印状态 JSON 消息

---

## 客户端实现参考

### JavaScript 示例

```javascript
// 1. 建立 WebSocket 连接
const ws = new WebSocket('ws://localhost:2024/print');
let sessionId = null;

// 2. 接收 sessionId
ws.onopen = function() {
  // sessionId 在首条消息中返回
};

ws.onmessage = function(event) {
  const data = JSON.parse(event.data);

  // 首个消息是 sessionId
  if (data.code === 0 && data.data) {
    sessionId = data.data;
    console.log('Session ID:', sessionId);
    // 此时可以发起打印请求
    return;
  }

  // 后续消息是打印状态
  switch (data.code) {
    case 200000: console.log('正在准备打印...'); break;
    case 200001: console.log('开始打印...'); break;
    case 200002: console.log('打印完成'); break;
    case 200003: console.error('打印异常:', data.msg); break;
    case 200004: console.log('数据传输完成'); break;
    case 200005: console.warn('需要人工干预'); break;
    case 200006: console.error('打印失败'); break;
    case 200007: console.log('打印已取消'); break;
    default: console.log('未知状态:', data);
  }
};

ws.onerror = function(error) {
  console.error('WebSocket 错误:', error);
};

ws.onclose = function() {
  console.log('WebSocket 连接已关闭');
};

// 3. 发起打印请求
async function print(data) {
  const formData = new FormData();
  formData.append('sessionId', sessionId);
  formData.append('type', data.type);
  formData.append('source', data.source);
  formData.append('content', data.content);
  if (data.copies) formData.append('copies', data.copies);
  if (data.color) formData.append('color', data.color);
  if (data.paperSize) formData.append('paperSize', data.paperSize);
  if (data.direction) formData.append('direction', data.direction);
  if (data.printServer) formData.append('printServer', data.printServer);

  const response = await fetch('http://localhost:2024/print/single', {
    method: 'POST',
    body: formData
  });
  return response.json();
}
```

---

## 注意事项

1. **sessionId 时效性**: 每次 WebSocket 连接或浏览器刷新都会生成新的 `sessionId`。打印请求必须使用当前有效连接的 `sessionId`，否则状态推送无法到达
2. **消息顺序**: PDF 打印的状态推送顺序为 `200000 → 200001 → 200004 → 200002`，IMG 打印为 `200000 → 200001`
3. **异常场景**: 异常状态码（`200003`）可能在任何阶段出现，客户端应随时处理异常状态
4. **线程安全**: 所有 WebSocket 操作基于 `CopyOnWriteArraySet` 和 `AtomicInteger`，天然线程安全
5. **消息格式**: 所有推送消息均为 JSON 字符串，反序列化为对象使用前建议做 `try-catch` 保护
6. **断线重连**: 客户端应实现重连机制，因为 WebSocket 断开后打印状态推送将丢失，且旧的 `sessionId` 作废
7. **HTML/TEXT 打印**: 这两种类型不推送任何状态，客户端仅依赖 HTTP 响应判断任务提交是否成功
