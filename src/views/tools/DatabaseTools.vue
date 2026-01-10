<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>数据库工具</h1>
      <p>数据库连接测试和查询</p>
    </div>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="['tab', { active: activeTab === tab.key }]"
        @click="activeTab = tab.key"
      >
        <i :class="tab.icon"></i> {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <!-- MySQL -->
      <div v-if="activeTab === 'mysql'" class="db-view">
        <div class="input-group">
          <div class="input-row">
            <div class="input-field">
              <label>主机:</label>
              <input v-model="mysqlConfig.host" type="text" class="text-field" placeholder="localhost" />
            </div>
            <div class="input-field">
              <label>端口:</label>
              <input v-model.number="mysqlConfig.port" type="number" class="text-field" placeholder="3306" />
            </div>
          </div>
          <div class="input-row">
            <div class="input-field">
              <label>用户名:</label>
              <input v-model="mysqlConfig.username" type="text" class="text-field" placeholder="root" />
            </div>
            <div class="input-field">
              <label>密码:</label>
              <input v-model="mysqlConfig.password" type="password" class="text-field" />
            </div>
          </div>
          <div class="input-field">
            <label>数据库:</label>
            <input v-model="mysqlConfig.database" type="text" class="text-field" placeholder="database_name" />
          </div>
          <div class="button-group">
            <button @click="testMysql" class="btn-action">测试连接</button>
          </div>
          <div v-if="mysqlResult" class="result">
            <div class="section-title">结果</div>
            <pre>{{ mysqlResult }}</pre>
          </div>
        </div>
      </div>

      <!-- PostgreSQL -->
      <div v-if="activeTab === 'postgres'" class="db-view">
        <div class="input-group">
          <div class="input-row">
            <div class="input-field">
              <label>主机:</label>
              <input v-model="postgresConfig.host" type="text" class="text-field" placeholder="localhost" />
            </div>
            <div class="input-field">
              <label>端口:</label>
              <input v-model.number="postgresConfig.port" type="number" class="text-field" placeholder="5432" />
            </div>
          </div>
          <div class="input-row">
            <div class="input-field">
              <label>用户名:</label>
              <input v-model="postgresConfig.username" type="text" class="text-field" placeholder="postgres" />
            </div>
            <div class="input-field">
              <label>密码:</label>
              <input v-model="postgresConfig.password" type="password" class="text-field" />
            </div>
          </div>
          <div class="input-field">
            <label>数据库:</label>
            <input v-model="postgresConfig.database" type="text" class="text-field" placeholder="database_name" />
          </div>
          <div class="button-group">
            <button @click="testPostgres" class="btn-action">测试连接</button>
          </div>
          <div v-if="postgresResult" class="result">
            <div class="section-title">结果</div>
            <pre>{{ postgresResult }}</pre>
          </div>
        </div>
      </div>

      <!-- SQLite -->
      <div v-if="activeTab === 'sqlite'" class="db-view">
        <div class="input-group">
          <div class="input-field">
            <label>数据库文件路径:</label>
            <input v-model="sqlitePath" type="text" class="text-field" placeholder="/path/to/database.db" />
          </div>
          <div class="button-group">
            <button @click="testSqlite" class="btn-action">测试连接</button>
          </div>
          <div v-if="sqliteResult" class="result">
            <div class="section-title">结果</div>
            <pre>{{ sqliteResult }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('mysql');
const tabs = [
  { key: 'mysql', name: 'MySQL', icon: 'fas fa-database' },
  { key: 'postgres', name: 'PostgreSQL', icon: 'fas fa-database' },
  { key: 'sqlite', name: 'SQLite', icon: 'fas fa-database' }
];

const mysqlConfig = ref({
  host: 'localhost',
  port: 3306,
  username: '',
  password: '',
  database: ''
});

const postgresConfig = ref({
  host: 'localhost',
  port: 5432,
  username: '',
  password: '',
  database: ''
});

const sqlitePath = ref('');
const mysqlResult = ref('');
const postgresResult = ref('');
const sqliteResult = ref('');

const testMysql = async () => {
  try {
    const result = await invoke<string>('test_mysql_connection', {
      host: mysqlConfig.value.host,
      port: mysqlConfig.value.port,
      username: mysqlConfig.value.username,
      password: mysqlConfig.value.password,
      database: mysqlConfig.value.database
    });
    mysqlResult.value = result;
    if ((window as any).$toast) {
      (window as any).$toast('MySQL连接成功', 'success');
    }
  } catch (e) {
    mysqlResult.value = (e as Error).message;
    if ((window as any).$toast) {
      (window as any).$toast('MySQL连接失败', 'error');
    }
  }
};

const testPostgres = async () => {
  try {
    const result = await invoke<string>('test_postgres_connection', {
      host: postgresConfig.value.host,
      port: postgresConfig.value.port,
      username: postgresConfig.value.username,
      password: postgresConfig.value.password,
      database: postgresConfig.value.database
    });
    postgresResult.value = result;
    if ((window as any).$toast) {
      (window as any).$toast('PostgreSQL连接成功', 'success');
    }
  } catch (e) {
    postgresResult.value = (e as Error).message;
    if ((window as any).$toast) {
      (window as any).$toast('PostgreSQL连接失败', 'error');
    }
  }
};

const testSqlite = async () => {
  try {
    const result = await invoke<string>('test_sqlite_connection', {
      filePath: sqlitePath.value
    });
    sqliteResult.value = result;
    if ((window as any).$toast) {
      (window as any).$toast('SQLite连接成功', 'success');
    }
  } catch (e) {
    sqliteResult.value = (e as Error).message;
    if ((window as any).$toast) {
      (window as any).$toast('SQLite连接失败', 'error');
    }
  }
};
</script>

<style scoped>
.tool-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.tool-header {
  margin-bottom: 30px;
}

.tool-header h1 {
  font-size: 28px;
  color: var(--text-primary);
  margin-bottom: 10px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.tab {
  padding: 12px 24px;
  border: none;
  border-radius: var(--border-radius);
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
  box-shadow: var(--shadow);
}

.tab.active {
  background: var(--primary);
  color: white;
}

.tab-content {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 25px;
  box-shadow: var(--shadow);
}

.input-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
  margin-bottom: 15px;
}

.input-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-field label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.text-field {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.button-group {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
  margin-top: 15px;
}

.btn-action {
  padding: 10px 20px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-action:hover {
  background: var(--secondary);
}

.result {
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  max-height: 300px;
  overflow: auto;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.result pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
