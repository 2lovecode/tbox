<script lang="ts" setup>
import { ref, computed } from 'vue'
import PageHeader from '@/components/PageHeader.vue';
import { toast } from '@/utils/toast';

interface PasswordEntry {
  id: string
  name: string
  username: string
  password: string
  url: string
  notes: string
  createdAt: string
}

const entries = ref<PasswordEntry[]>([])
const searchQuery = ref('')
const showAddForm = ref(false)
const showPassword = ref<Record<string, boolean>>({})
const editingId = ref<string | null>(null)

const newEntry = ref({
  name: '',
  username: '',
  password: '',
  url: '',
  notes: ''
})

const filteredEntries = computed(() => {
  if (!searchQuery.value) return entries.value
  const query = searchQuery.value.toLowerCase()
  return entries.value.filter(entry => 
    entry.name.toLowerCase().includes(query) ||
    entry.username.toLowerCase().includes(query) ||
    entry.url.toLowerCase().includes(query)
  )
})

// 生成强密码
const generatePassword = (length: number = 16) => {
  const charset = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*'
  let password = ''
  for (let i = 0; i < length; i++) {
    password += charset.charAt(Math.floor(Math.random() * charset.length))
  }
  newEntry.value.password = password
}

// 添加/更新密码条目
const saveEntry = () => {
  if (!newEntry.value.name || !newEntry.value.password) {
    toast.warning('请填写名称和密码')
    return
  }

  if (editingId.value) {
    // 更新
    const index = entries.value.findIndex(e => e.id === editingId.value)
    if (index !== -1) {
      entries.value[index] = {
        ...entries.value[index],
        ...newEntry.value
      }
    }
    editingId.value = null
  } else {
    // 新增
    entries.value.push({
      id: Date.now().toString(),
      ...newEntry.value,
      createdAt: new Date().toISOString()
    })
  }

  // 保存到本地存储 - 添加错误处理
  try {
    localStorage.setItem('passwordEntries', JSON.stringify(entries.value))
    toast.success('保存成功')
  } catch (error) {
    console.error('保存失败:', error)
    toast.error('保存失败，可能是存储空间不足')
    return
  }

  // 重置表单
  newEntry.value = {
    name: '',
    username: '',
    password: '',
    url: '',
    notes: ''
  }
  showAddForm.value = false
}

// 编辑条目
const editEntry = (entry: PasswordEntry) => {
  newEntry.value = { ...entry }
  editingId.value = entry.id
  showAddForm.value = true
}

// 删除条目
const deleteEntry = (id: string) => {
  if (confirm('确定要删除这条密码记录吗？')) {
    entries.value = entries.value.filter(e => e.id !== id)
    try {
      localStorage.setItem('passwordEntries', JSON.stringify(entries.value))
      toast.success('删除成功')
    } catch (error) {
      console.error('删除失败:', error)
      toast.error('删除失败')
    }
  }
}

// 复制到剪贴板
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('已复制到剪贴板')
  } catch (err) {
    console.error('复制失败:', err)
    toast.error('复制失败')
  }
}

// 切换密码显示
const togglePassword = (id: string) => {
  showPassword.value[id] = !showPassword.value[id]
}

// 加载保存的密码 - 添加错误处理
const loadEntries = () => {
  try {
    const saved = localStorage.getItem('passwordEntries')
    if (saved) {
      const parsed = JSON.parse(saved)
      if (Array.isArray(parsed)) {
        entries.value = parsed
      }
    }
  } catch (error) {
    console.error('加载密码记录失败:', error)
    toast.error('加载密码记录失败')
    entries.value = []
  }
}

// 初始化
loadEntries()
</script>

<template>
  <div class="password-manage">
    <PageHeader title="密码管理器" :show-back="true" />
    
    <div class="header-actions">
      <button @click="showAddForm = true" class="add-btn">
        <i class="fas fa-plus"></i> 添加密码
      </button>
    </div>

    <div class="search-bar">
      <i class="fas fa-search"></i>
      <input 
        type="text" 
        v-model="searchQuery"
        placeholder="搜索密码记录..."
      />
    </div>

    <!-- 添加/编辑表单 -->
    <div v-if="showAddForm" class="form-modal">
      <div class="form-content">
        <h2>{{ editingId ? '编辑密码' : '添加密码' }}</h2>
        <div class="form-group">
          <label>名称 *</label>
          <input v-model="newEntry.name" type="text" placeholder="例如：GitHub" />
        </div>
        <div class="form-group">
          <label>用户名/邮箱</label>
          <input v-model="newEntry.username" type="text" placeholder="用户名或邮箱" />
        </div>
        <div class="form-group">
          <label>密码 *</label>
          <div class="password-input-group">
            <input 
              v-model="newEntry.password" 
              :type="showPassword['new'] ? 'text' : 'password'" 
              placeholder="密码"
            />
            <button @click="showPassword['new'] = !showPassword['new']" class="toggle-password">
              <i :class="showPassword['new'] ? 'fas fa-eye-slash' : 'fas fa-eye'"></i>
            </button>
            <button @click="generatePassword()" class="generate-btn">
              <i class="fas fa-key"></i> 生成
            </button>
          </div>
        </div>
        <div class="form-group">
          <label>网址</label>
          <input v-model="newEntry.url" type="url" placeholder="https://example.com" />
        </div>
        <div class="form-group">
          <label>备注</label>
          <textarea v-model="newEntry.notes" placeholder="备注信息"></textarea>
        </div>
        <div class="form-actions">
          <button @click="saveEntry()" class="save-btn">保存</button>
          <button @click="showAddForm = false; editingId = null; newEntry = { name: '', username: '', password: '', url: '', notes: '' }" class="cancel-btn">取消</button>
        </div>
      </div>
    </div>

    <!-- 密码列表 -->
    <div class="entries-list">
      <div v-if="filteredEntries.length === 0" class="empty-state">
        <i class="fas fa-lock"></i>
        <p>{{ searchQuery ? '没有找到匹配的密码记录' : '还没有保存任何密码' }}</p>
      </div>

      <div v-for="entry in filteredEntries" :key="entry.id" class="entry-card">
        <div class="entry-header">
          <h3>{{ entry.name }}</h3>
          <div class="entry-actions">
            <button @click="editEntry(entry)" class="icon-btn" title="编辑">
              <i class="fas fa-edit"></i>
            </button>
            <button @click="deleteEntry(entry.id)" class="icon-btn" title="删除">
              <i class="fas fa-trash"></i>
            </button>
          </div>
        </div>
        <div class="entry-body">
          <div v-if="entry.username" class="entry-field">
            <label>用户名:</label>
            <span>{{ entry.username }}</span>
            <button @click="copyToClipboard(entry.username)" class="copy-btn">
              <i class="fas fa-copy"></i>
            </button>
          </div>
          <div class="entry-field">
            <label>密码:</label>
            <span>{{ showPassword[entry.id] ? entry.password : '••••••••' }}</span>
            <button @click="togglePassword(entry.id)" class="toggle-btn">
              <i :class="showPassword[entry.id] ? 'fas fa-eye-slash' : 'fas fa-eye'"></i>
            </button>
            <button @click="copyToClipboard(entry.password)" class="copy-btn">
              <i class="fas fa-copy"></i>
            </button>
          </div>
          <div v-if="entry.url" class="entry-field">
            <label>网址:</label>
            <a :href="entry.url" target="_blank" class="url-link">{{ entry.url }}</a>
          </div>
          <div v-if="entry.notes" class="entry-field">
            <label>备注:</label>
            <span>{{ entry.notes }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.password-manage {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0;
}

.header-actions {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 20px;
}

.add-btn {
  background: var(--primary);
  color: white;
  border: none;
  padding: 12px 24px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 16px;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
}

.add-btn:hover {
  background: var(--secondary);
}

.search-bar {
  position: relative;
  margin-bottom: 30px;
}

.search-bar i {
  position: absolute;
  left: 15px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--gray);
}

.search-bar input {
  width: 100%;
  padding: 12px 15px 12px 45px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
}

.form-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.form-content {
  background: white;
  padding: 30px;
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow-y: auto;
}

.form-content h2 {
  margin-bottom: 20px;
  color: var(--dark);
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--dark);
}

.form-group input,
.form-group textarea {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.form-group textarea {
  min-height: 80px;
  resize: vertical;
}

.password-input-group {
  display: flex;
  gap: 8px;
}

.password-input-group input {
  flex: 1;
}

.toggle-password,
.generate-btn {
  padding: 10px 15px;
  border: 1px solid #ddd;
  background: white;
  border-radius: 6px;
  cursor: pointer;
  transition: var(--transition);
}

.toggle-password:hover,
.generate-btn:hover {
  background: #f5f5f5;
}

.form-actions {
  display: flex;
  gap: 10px;
  margin-top: 20px;
}

.save-btn,
.cancel-btn {
  flex: 1;
  padding: 12px;
  border: none;
  border-radius: 6px;
  font-size: 16px;
  cursor: pointer;
  transition: var(--transition);
}

.save-btn {
  background: var(--primary);
  color: white;
}

.save-btn:hover {
  background: var(--secondary);
}

.cancel-btn {
  background: #f5f5f5;
  color: var(--dark);
}

.cancel-btn:hover {
  background: #e0e0e0;
}

.entries-list {
  display: grid;
  gap: 20px;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--gray);
}

.empty-state i {
  font-size: 64px;
  margin-bottom: 20px;
  opacity: 0.5;
}

.entry-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: var(--transition);
}

.entry-card:hover {
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
}

.entry-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  padding-bottom: 15px;
  border-bottom: 1px solid #eee;
}

.entry-header h3 {
  font-size: 20px;
  color: var(--dark);
}

.entry-actions {
  display: flex;
  gap: 10px;
}

.icon-btn {
  background: none;
  border: none;
  color: var(--gray);
  cursor: pointer;
  padding: 8px;
  border-radius: 6px;
  transition: var(--transition);
}

.icon-btn:hover {
  background: #f5f5f5;
  color: var(--primary);
}

.entry-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.entry-field {
  display: flex;
  align-items: center;
  gap: 10px;
}

.entry-field label {
  font-weight: 500;
  min-width: 80px;
  color: var(--gray);
}

.entry-field span {
  flex: 1;
  word-break: break-all;
}

.copy-btn,
.toggle-btn {
  background: none;
  border: none;
  color: var(--primary);
  cursor: pointer;
  padding: 5px 10px;
  border-radius: 4px;
  transition: var(--transition);
}

.copy-btn:hover,
.toggle-btn:hover {
  background: rgba(67, 97, 238, 0.1);
}

.url-link {
  color: var(--primary);
  text-decoration: none;
  word-break: break-all;
}

.url-link:hover {
  text-decoration: underline;
}
</style>
