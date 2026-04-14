# 可视化已读与未读位置跳转 (Visual Read Receipts) 实施计划

本计划旨在完美实现微信群聊中“定位到上次读到的消息位置，随着消息曝光在屏幕内再标记已读”的功能。
为了绝对的安全和防错，整个功能拆解为 **4 个可独立验证的里程碑步骤**，遵循“先底层、后接口、最后页面”的原则逐步推进。

---

## 🏗️ 步骤一：数据库与数据实体层的准备 (Backend Data Layer)
我们要加入 `read_msg_id` 从而把房间的绝对进度和个人的阅读进度分离。

1. **执行建表/改表 SQL (手动执行)**：
   * `ALTER TABLE contact ADD COLUMN read_msg_id BIGINT NULL COMMENT '最后一次已读的消息ID' AFTER last_msg_id;`
2. **修改 Rust 实体定义 (`contact/model/entity.rs`)**：
   * 在 `Contact` 结构体中新增 `pub read_msg_id: Option<i64>,` 映射数据库。
3. **修改 DTO 传输层 (`contact/model/dto.rs` / `im.ts`)**：
   * 让 `ContactVO` 对外暴露 `read_msg_id: Option<i64>`。
   * **验证要求：** 修改后微服务编译通过，前端在未读列表能打印出 `read_msg_id`。

---

## ⚙️ 步骤二：后端核心业务流重构 (Backend Core Logic)
这是后端的精髓部分，需要解耦原有的强绑定已读逻辑，并丰富游标查询能力。

1. **阻断旧的“强塞已读”逻辑 (`message/repository.rs`)**：
   * 在 `update_contacts_active` 方法中，当别人发消息给你时，**绝不要**再修改接收方的 `last_msg_id` 和 `read_msg_id`，只准更新 `unread_count = unread_count + 1` 和 `active_time`。（保证游标不乱跑）。
2. **扩充双向游标分页 (`message/service.rs`)**：
   * 修改 `list_messages` 方法，支持 `query.fetch_mode` (前端传 0=历史向下看，1=未读向上看)。
   * 当前端要看未读新消息时，SQL 从 `WHERE id < cursor ORDER BY id DESC` 变成 `WHERE id > cursor ORDER BY id ASC`。
3. **重写精准标记已读接口 (`contact/repository.rs` & `handler`)**：
   * 废弃原有的 `update_read_time` 全量清零。
   * 新增 API：`POST /im/contact/mark-read` 接受参数 `room_id` 和 `max_msg_id`。
   * SQL 实现：`UPDATE contact SET read_msg_id = ?, unread_count = GREATEST(0, unread_count - ?) WHERE room_id = ? AND uid = ? AND (read_msg_id IS NULL OR read_msg_id < ?)`，原位精准扣减确保并发安全。
   * **验证要求：** Postman 或 Swagger 调用标记已读后，数据库对应的 `read_msg_id` 和 `unread_count` 发生正确的定量变化。

---

## 🔄 步骤三：前端状态管理改造 (Frontend Store)
这步让前端能够支持正反双向获取聊天记录，并正确维系内存状态。

1. **改进 `contact.ts` / `chat.ts` 状态机**：
   * 在 Vue 的 `chatStore` 中修改 `loadMessages`，使其能接收 `read_msg_id` 并在进房时调用新的“正向查询” API。
   * 如果进房时 `contact.unread_count > 0` 且 `contact.read_msg_id` 存在，则**放弃默认的拉取最新 20 条**，改为以 `read_msg_id` 为圆心，拉取附近的 20-30 条上下文。
2. **本地内存排序接管**：
   * 由于拉未读消息是 `ASC` 顺序，前端需要兼容拼接逻辑，把数据原样倒置压入列表，确保在聊天窗口呈现出按时间排序好的连续消息序列。
   * **验证要求：** 前端在控制台中能够正确组装出含有上次看过的消息，以及接下来待看消息的长列表内存对象，暂时不关心界面表现。

---

## 🎨 步骤四：前端进房定位与曝光检测 (Frontend DOM & UX)
这是最后一步，也就是最极致的交互体验。

1. **DOM 锚点渲染与瞬间定位 (`Message.vue` / `ChatRoom.vue`)**：
   * 模板判断：在 `v-for` 中如果碰到了 `msg.id === read_msg_id` 的那一条记录，立刻在上方渲染一条 HTML 分割线 `<div class="divider">—— 以下为新消息 ——</div>`。
   * 进房滚动：在 `onMounted` 和 `nextTick` 中，获取该分割线元素的 DOM，执行 `scrollIntoView({ block: 'center', behavior: 'instant' })`。
2. **`v-intersect` 视野曝光收集**：
   * 为所有的消息气泡编写一个 Vue 的自定义指令，或者使用 `IntersectionObserver` 钩子。
   * 监听所有在其上方的分割线以后的新消息气泡，当它们滚动到屏幕内可见度达 50% 时，将其 `msg.id` 读取并更新本地的一个防抖缓冲 `max_visible_id`。
3. **防抖异步提交**：
   * 每隔 1 500ms，只需用这个最新算出来的 `max_visible_id` 以及已读的偏移量，静默调用后端的精准 `mark_read` 接口即可，完全不闪烁不卡顿。
   * **验证要求：** 完整跑通微信同款的进群定位、滚动慢慢吃掉未读角标量的极致体验！
