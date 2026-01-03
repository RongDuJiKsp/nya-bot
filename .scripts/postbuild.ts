import { readFile, writeFile } from 'fs/promises';
import path from 'path';
import process from 'process';
import toml from '@iarna/toml';

function dockerEnv(name: string) {
  const val = process.env[name];
  if (!val) {
    throw new Error(`Docker 的上下文变量 ${name} 未指定。请确保 build 脚本由 docker 构建脚本 运行`);
  }
  return val;
}
// 配置文件名
const KOVI_CONF_TEMPLATE_NAME = dockerEnv('KOVI_CONF_TEMPLATE_NAME');
const KOVI_CONF_USE_NAME = dockerEnv('KOVI_CONF_USE_NAME');



async function cKoviConf() {
  if (!process.env.FOR_CMD) {
    console.info(`未指定 FOR_CMD 或解析失败，跳过生成 ${KOVI_CONF_USE_NAME}`);
    return;
  }
  const workDir = dockerEnv('SCRIPT_WORK_DIR');

  const configPath = path.join(workDir, KOVI_CONF_TEMPLATE_NAME);
  const content = await readFile(configPath, 'utf-8');

  // 解析 TOML
  const doc:Record<string, any> = toml.parse(content);

  // 修改 [config].main_admin
  // 原模板注释: "# 管理员 ID"
  if (process.env.MAIN_ADMIN) {
    doc.config = doc.config || {};
    doc.config.main_admin = Number(process.env.MAIN_ADMIN);
  }

  // 修改 [config].admins
  // 原模板注释: "# 管理员列表"
  if (process.env.ADMINS) {
    doc.config = doc.config || {};
    doc.config.admins = process.env.ADMINS.split(',').map((x) => Number(x));
  }

  // 修改 [server].host
  // 原模板注释: "# 服务器地址"
  if (process.env.HOST) {
    doc.server = doc.server || {};
    doc.server.host = process.env.HOST;
  }

  // 修改 [server].port
  // 原模板注释: "# 服务器端口"
  if (process.env.PORT) {
    doc.server = doc.server || {};
    doc.server.port = Number(process.env.PORT);
  }

  // 修改 [server].access_token
  // 原模板注释: "# 访问令牌"
  if (process.env.ACCESS_TOKEN) {
    doc.server = doc.server || {};
    doc.server.access_token = process.env.ACCESS_TOKEN;
  }

  // 修改 [server].secure
  // 原模板注释: "# 是否启用安全模式"
  if (process.env.SECURE) {
    doc.server = doc.server || {};
    doc.server.secure = process.env.SECURE.length > 0;
  }

  // 输出到 OUT_DIR
  const outPath = path.join(workDir, KOVI_CONF_USE_NAME);
  const outputContent = toml.stringify(doc);
  await writeFile(outPath, outputContent, 'utf-8');
}

// 入口
async function main(){
  await cKoviConf();
}
main().catch(console.error);
