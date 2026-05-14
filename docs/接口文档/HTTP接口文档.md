# Java Print HTTP 接口文档

## 概述

Java Print 是一个基于 Spring Boot 的打印服务中间件，提供 HTTP REST API 供客户端调用，实现打印机列表查询、内容打印、条码打印、模板资源获取等功能。

- **基础路径**: `http://{host}:{port}`
- **默认端口**: `2024`（可通过 `setting.xml` 配置 `serverport` 节点修改）
- **请求编码**: `UTF-8`
- **跨域支持**: 已全局开启 CORS，允许所有来源、所有常用 HTTP 方法

---

## 通用响应格式

所有 `/print` 路径下的接口均返回统一响应体 `CommonResult<T>`，JSON 格式如下：

| 字段 | 类型 | 说明 |
|------|------|------|
| `code` | `Integer` | 状态码，`0` 表示成功，非 `0` 表示失败 |
| `msg` | `String` | 状态码对应的提示信息 |
| `data` | `T`（泛型） | 响应数据，具体结构视接口而定；失败时可能为 `null` |

**示例（成功）**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": [...]
}
```

**示例（失败）**:
```json
{
  "code": 1,
  "msg": "操作失败"
}
```

---

## 全局状态码

| 状态码 | 含义 |
|--------|------|
| `0` | 操作成功 |
| `1` | 操作失败 |
| `100001` | 参数不正确 |
| `200000` | 开始准备 |
| `200001` | 开始打印 |
| `200002` | 打印结束 |
| `200003` | 打印异常 |
| `200004` | 数据传输完成 |
| `200005` | 需要人工干预 |
| `200006` | 打印失败 |
| `200007` | 打印取消 |
| `200008` | 系统找不到指定的文件 |
| `200009` | 文件异常 |

> 注：`200000` ~ `200009` 为打印过程中通过 WebSocket 推送的状态码，HTTP 响应通常只返回 `0`（成功）或 `1`（失败）。

---

## 接口列表

### 1. 获取打印机列表

获取当前客户端系统上所有可用的打印机名称列表。

- **URL**: `/print/getPrintServers`
- **Method**: `GET`
- **Content-Type**: 无需指定

**请求参数**: 无

**响应示例**:
```json
{
  "code": 0,
  "msg": "操作成功",
  "data": [
    "HP LaserJet Pro M404dn",
    "Microsoft Print to PDF",
    "Fax",
    "\\\\192.168.1.100\\SharedPrinter"
  ]
}
```

**响应字段说明**:

| 字段 | 类型 | 说明 |
|------|------|------|
| `data` | `String[]` | 打印机名称字符串数组，每个元素为系统注册的打印服务名称 |

**注意事项**:
- 该接口返回的打印机名称可直接作为 [`/print/single`](#2-单个打印核心接口) 接口中 `printServer` 参数的值，用于指定打印机
- 网络共享打印机的名称格式为 `\\主机名\共享名`

---

### 2. 单个打印（核心接口）

根据传入的打印参数，将内容发送到指定打印机进行打印。支持 PDF、图片、文本、HTML 四种打印类型，支持四种内容来源。

- **URL**: `/print/single`
- **Method**: `POST`
- **Content-Type**: `multipart/form-data`

#### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `type` | `String` | **是** | 打印类型，决定打印内容的处理方式 |
| `source` | `String` | **是** | 内容来源类型，决定如何获取打印内容 |
| `content` | `String` | 条件必填 | 打印内容，具体含义取决于 `source` 参数 |
| `file` | `MultipartFile` | 条件必填 | 文件流，仅当 `source=blob` 时使用 |
| `sessionId` | `String` | 推荐 | WebSocket 会话 ID，用于接收打印状态推送 |
| `copies` | `Integer` | 否 | 打印份数，缺省为 `1` |
| `color` | `Boolean` | 否 | 是否彩色打印，`true`=彩色，`false`=黑白。缺省为 `false` |
| `paperSize` | `String` | 否 | 纸张尺寸，缺省时由打印机默认设置决定 |
| `direction` | `String` | 否 | 打印方向，缺省为纵向 |
| `printServer` | `String` | 否 | 指定打印机名称，缺省时使用系统默认打印机 |

#### 3.1 `type` 参数枚举值

| 值 | 说明 | 支持的 source |
|----|------|---------------|
| `PDF` | PDF 文档打印，使用 PDFBox 渲染 | `path`, `url`, `blob` |
| `IMG` | 图片打印，支持 JPEG 格式 | `path`, `url`, `blob` |
| `TEXT` | 纯文本打印 | `text` |
| `HTML` | HTML 内容打印 | `text` |

#### 3.2 `source` 参数枚举值

| 值 | 说明 | `content` 字段含义 | 额外参数 |
|----|------|---------------------|----------|
| `text` | 纯文本/HTML 字符串 | 直接传入要打印的文本或 HTML 内容 | 无 |
| `path` | 本地文件路径 | 文件的绝对路径，如 `D:\docs\report.pdf` | 无 |
| `url` | 网络地址 | 可访问的文件 URL，如 `https://example.com/doc.pdf` | 无 |
| `blob` | 文件流（二进制） | 不使用 `content` | `file` 参数必填 |

#### 3.3 `paperSize` 参数枚举值

| 值 | 对应纸张规格 |
|----|--------------|
| `A` | A 系列（通用） |
| `B` | B 系列（通用） |
| `C` | C 系列（通用） |
| `D` | D 系列（通用） |
| `E` | E 系列（通用） |
| `EXECUTIVE` | Executive (7.25 × 10.5 in) |
| `FOLIO` | Folio |
| `INVOICE` | Invoice (5.5 × 8.5 in) |
| `ISO_A0` | A0 (841 × 1189 mm) |
| `ISO_A1` | A1 (594 × 841 mm) |
| `ISO_A2` | A2 (420 × 594 mm) |
| `ISO_A3` | A3 (297 × 420 mm) |
| `ISO_A4` | A4 (210 × 297 mm) |
| `ISO_A5` | A5 (148 × 210 mm) |
| `ISO_A6` | A6 (105 × 148 mm) |
| `ISO_A7` | A7 (74 × 105 mm) |
| `ISO_A8` | A8 (52 × 74 mm) |
| `ISO_A9` | A9 (37 × 52 mm) |
| `ISO_A10` | A10 (26 × 37 mm) |

#### 3.4 `direction` 参数枚举值

| 值 | 说明 |
|----|------|
| `PORTRAIT` | 纵向（默认方向） |
| `LANDSCAPE` | 横向（逆时针旋转 90°） |
| `REVERSE_LANDSCAPE` | 反向横向（顺时针旋转 90°） |
| `REVERSE_PORTRAIT` | 反向纵向（旋转 180°） |

#### 请求示例

**示例 1: 打印本地 PDF 文件**
```
POST /print/single
Content-Type: multipart/form-data

type=PDF
source=path
content=D:\docs\report.pdf
sessionId=abc123-def456
copies=2
color=false
paperSize=ISO_A4
direction=PORTRAIT
printServer=HP LaserJet Pro M404dn
```

**示例 2: 通过 URL 打印图片**
```
POST /print/single
Content-Type: multipart/form-data

type=IMG
source=url
content=https://example.com/barcode.jpg
sessionId=abc123-def456
color=true
printServer=
```

**示例 3: 上传文件流打印 PDF**
```
POST /print/single
Content-Type: multipart/form-data

type=PDF
source=blob
file=@/path/to/local/document.pdf
sessionId=abc123-def456
copies=1
```

**示例 4: 打印纯文本内容**
```
POST /print/single
Content-Type: multipart/form-data

type=TEXT
source=text
content=这是一段要打印的文本内容
sessionId=abc123-def456
```

#### 响应

**成功响应**:
```json
{
  "code": 0,
  "msg": "操作成功"
}
```

**失败响应（不支持的打印类型）**:
```json
{
  "code": 1,
  "msg": "操作失败"
}
```
> 注：实际异常详情会在服务端控制台输出（如 `IllegalArgumentException: 不支持的打印类型: XXX`），HTTP 响应统一返回失败状态码。

#### 重要说明

1. **打印为异步执行**: 接口立即返回，实际打印在后台线程池中执行（最大 10 个并发线程）
2. **打印状态推送**: 通过 `sessionId` 参数关联 WebSocket 连接，打印过程中的状态变化（开始准备、开始打印、打印结束、异常等）会通过 WebSocket 实时推送给对应客户端
3. **source=blob 的文件限制**: 最大上传文件大小为 `10MB`，最大请求大小为 `50MB`（由 Spring `multipart` 配置限定）
4. **指定打印机**: 若 `printServer` 为空或未传，则使用系统默认打印机；若指定了名称但系统中不存在，会导致打印失败
5. **PDF/图片类型**: 不支持 `source=text`（纯文本方式），会抛出 `UnsupportedOperationException`
6. **HTML 类型**: 当前仅输出日志，实际打印功能尚未完整实现

---

### 3. 获取程序运行路径

获取服务端程序的当前工作目录路径，结果通过 WebSocket 推送给指定会话。

- **URL**: `/print/getPath/{sessionId}`
- **Method**: `GET`

#### 路径参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `sessionId` | `String` | **是** | WebSocket 会话 ID，路径信息将推送到该会话 |

#### 请求示例
```
GET /print/getPath/abc123-def456
```

#### 响应
- HTTP 响应体为空（`void`），不通过 HTTP 返回数据
- 路径信息通过 WebSocket 以纯文本字符串形式推送到指定 `sessionId`

#### 说明
返回的是 `System.getProperty("user.dir")` 的值，即 Java 进程的工作目录，通常为程序启动目录。

---

### 4. 条码打印（北洋打印机）

调用北洋（SNBC）条码打印机 SDK 进行标签条码打印。通过反射加载第三方 JAR 包实现。

- **URL**: `/print/barCodePrint`
- **Method**: `POST`
- **Content-Type**: `multipart/form-data`

#### 请求参数

| 参数名 | 类型 | 必填 | 说明 |
|--------|------|------|------|
| `checkName` | `String` | **是** | 检测项目名称，打印在标签的检测名称区域 |
| `code` | `String` | **是** | 条码编码内容，将编码为 CODE128 一维条码 |
| `spec` | `String` | **是** | 规格型号，如 `HRB400E φ 初检`，打印在标签的规格区域 |

#### 请求示例
```
POST /print/barCodePrint
Content-Type: multipart/form-data

checkName=钢筋原材
code=GJYC-2024-001
spec=HRB400E Ø 初检
```

#### 打印标签布局

| 区域 | 位置 | 内容 | 字体 |
|------|------|------|------|
| 检测名称 | 横坐标 20, 纵坐标 200 | `checkName` 参数值 | P18, 15pt, 粗体, 宋体 |
| 规格型号 | 横坐标 20, 纵坐标 170 | `spec` 参数值 | P18, 15pt, 粗体, 宋体 |
| 条码 | 横坐标 40, 纵坐标 40 | `code` 参数值（CODE128 编码） | 条码高度 80, 注释在下方 |
| 状态 | 横坐标 20, 纵坐标 20 | 固定文本 "状态:待检√在检_已检_备份_" | P18, 15pt, 粗体, 宋体 |

#### 标签设置

| 参数 | 值 |
|------|-----|
| 标签宽度 | 450 |
| 标签高度 | 250 |
| 打印模式 | 3（回卷模式），出纸口距离 250 |
| 连接方式 | BPLA，端口 6 |

#### 响应
- HTTP 响应体为空（`void`），不返回 JSON 数据
- 打印结果通过服务端控制台日志输出

#### 前提条件
- 需要在程序同级目录下放置 `libs/LabelPrinterJavaSDK.jar` 文件
- 需要北洋打印机通过 USB 连接到客户端机器

---

### 5. 代理转发（通配路由）

转发所有未被其他 Controller 精确匹配的 HTTP 请求到配置的代理目标地址。

- **URL**: `/**`（匹配所有路径）
- **Method**: `GET`, `POST`, `PUT`, `DELETE`, `PATCH`
- **Content-Type**: 透传

#### 说明

1. 此路由优先级最低，仅当请求 URL 不匹配任何其他 Controller 的精确路径时才生效
2. 当代理类型为 `2`（网页代理 HTTP）时，将请求转发到 `http://{domain}:{port}/` + 原始请求 URI
3. 当代理类型为 `1`（不使用代理）时，`targetUrl` 为空字符串，请求转到 `http://{requestURI}`（通常不可达）
4. 请求头、请求体、HTTP 方法均透传，响应也透传回客户端
5. `/favicon.ico` 请求会被直接拦截并返回空响应，避免干扰

#### 代理配置

代理类型在 `setting.xml` 的 `<proxy>` 节点中配置：

| 配置项 | XML 属性 | 可选值 | 说明 |
|--------|----------|--------|------|
| 代理类型 | `type` | `1` / `2` | `1`=不使用代理，`2`=网页代理（HTTP） |
| 代理地址 | `domain` | IP/域名 | 代理目标主机地址 |
| 代理端口 | `port` | 端口号 | 代理目标端口 |

---

### 6. 模板资源接口

以下接口用于获取打印模板的 HTML/CSS 资源文件，返回文件内容字符串供前端渲染打印模板。

#### 6.1 委托模版

- **URL**: `/powerAttorney/{filename}`
- **Method**: `GET`

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 委托模版文件名，如 `template1.html` |

- **响应**: `text/html;charset=UTF-8` — 文件内容字符串
- **资源目录**: `{user.dir}\public\powerAttorney\`

#### 6.2 报告模版

- **URL**: `/report/{filename}`
- **Method**: `GET`

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 报告模版文件名 |

- **响应**: 文件内容字符串 (UTF-8)
- **资源目录**: `{user.dir}\public\report\`

#### 6.3 原始记录模版

- **URL**: `/originalRecords/{filename}`
- **Method**: `GET`

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 原始记录模版文件名 |

- **资源目录**: `{user.dir}\public\originalRecords\`

#### 6.4 不合格报告模版

- **URL**: `/public/unquaLedger/{filename}`
- **Method**: `GET`

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 不合格报告模版文件名 |

- **资源目录**: `{user.dir}\public\unquaLedger\`

#### 6.5 封皮模版

- **URL**: `/cover/{filename}`
- **Method**: `GET`

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 封皮模版文件名 |

- **资源目录**: `{user.dir}\public\cover\`

#### 6.6 费用列表模版

- **URL**: `/feeList/{filename}`
- **Method**: `GET`

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 费用列表模版文件名 |

- **资源目录**: `{user.dir}\public\feeList\`

---

### 7. 字体资源

提供打印所需的字体文件下载及可用字体列表查询。

- **URL**: `/fonts/{filename}`
- **Method**: `GET`

#### 请求参数

| 参数 | 类型 | 位置 | 必填 | 说明 |
|------|------|------|------|------|
| `filename` | `String` | Path | **是** | 字体文件名。传入 `.ttf` 后缀的文件名获取字体文件；传入任意非 `.ttf` 的参数获取字体列表 JSON |

#### 响应说明

**场景 1 — `filename` 以 `.ttf` 结尾（获取字体文件）**:

| 响应头 | 值 |
|--------|-----|
| `Content-Type` | `application/octet-stream` |
| `Content-Length` | 文件字节数 |

返回字体文件的二进制内容。资源目录: `{user.dir}\public\fonts\`

**场景 2 — `filename` 不以 `.ttf` 结尾（获取字体列表）**:

返回固定 JSON:
```json
{
  "path": "",
  "descriptors": [
    {
      "name": "微软雅黑",
      "source": "../fonts/SimHei.ttf"
    },
    {
      "name": "宋体",
      "source": "../fonts/song.ttf"
    },
    {
      "name": "黑体",
      "source": "../fonts/SimHei.ttf"
    }
  ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `path` | `String` | 路径前缀，当前为空 |
| `descriptors` | `Array` | 字体描述数组 |
| `descriptors[].name` | `String` | 字体的显示名称 |
| `descriptors[].source` | `String` | 字体文件的相对路径 |

---

## 错误处理

### HTTP 层面

| HTTP 状态码 | 场景 |
|-------------|------|
| `200` | 正常响应（业务是否成功需检查 `code` 字段） |
| `400` | 请求参数解析失败（如必填参数缺失、类型不匹配） |
| `404` | 请求路径不存在 |
| `405` | 请求方法不允许 |
| `413` | 上传文件大小超过限制 |
| `500` | 服务端内部异常 |

### 全局异常说明

| 异常 | 说明 |
|------|------|
| `IllegalArgumentException` | 不支持的 `type`、`source`、`paperSize` 或 `direction` 值 |
| `UnsupportedOperationException` | IMG/PDF 打印服务不支持 `source=text` |
| `FileNotFoundException` | 模板文件（`path` 来源）或资源文件不存在 |
| `MalformedURLException` | `url` 来源的 URL 格式不正确 |

---

## 配置参考

### application.properties
```properties
server.port=2024                                    # 服务端口
spring.servlet.multipart.max-file-size=10MB         # 单文件上传最大限制
spring.servlet.multipart.max-request-size=50MB      # 单次请求最大限制
spring.servlet.multipart.location=/temp/dir4         # 上传临时目录
```

### setting.xml 关键节点

```xml
<setting>
  <proxy>
    <content type="1" domain="" port="" />
  </proxy>
  <serverport>
    <content port="2024" />
  </serverport>
  <backimg>
    <content path="" />
  </backimg>
</setting>
```

| 节点 | 属性 | 说明 |
|------|------|------|
| `proxy > content` | `type` | 代理类型：`1`=不使用，`2`=HTTP 代理 |
| `proxy > content` | `domain` | 代理目标域名/IP |
| `proxy > content` | `port` | 代理目标端口 |
| `serverport > content` | `port` | 服务端口号（优先级高于 `application.properties`） |
| `backimg > content` | `path` | 背景图片路径 |

---

## CORS 配置

所有接口已全局开启跨域支持：

| 配置项 | 值 |
|--------|-----|
| 允许路径 | `/**` |
| 允许来源 | `*`（所有域名） |
| 允许方法 | `GET`, `POST`, `PUT`, `DELETE`, `OPTIONS` |
| 允许请求头 | `*`（所有请求头） |
| 允许凭证 | `false` |
| 预检缓存 | 3600 秒 |
