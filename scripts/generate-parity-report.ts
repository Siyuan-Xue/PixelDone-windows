import { loadManifest, summarize } from './parity';

const manifest = await loadManifest();
const summary = summarize(manifest);
const rows = manifest.rows
  .map((row) => `| ${row.id} | ${row.area} | ${row.title} | ${row.requiredForRelease ? '是' : '否'} | ${row.status} | ${row.variance} |`)
  .join('\n');
const markdown = `# PixelDone Windows 功能复刻报告

> 本文件由 \`bun run parity:report\` 从 \`parity/pixeldone-3.1.0.yaml\` 生成，请勿手工维护状态。

基线：PixelDone Android ${manifest.baseline.version}（versionCode ${manifest.baseline.versionCode}，commit \`${manifest.baseline.commit}\`，Room v${manifest.baseline.roomSchema}）。

- Required：${summary.required}
- Verified：${summary.counts.verified}
- In progress：${summary.counts.in_progress}
- Blocked：${summary.counts.blocked}
- Not started：${summary.counts.not_started}
- 完成率：${summary.percent.toFixed(2)}%

| ID | 域 | 功能 | Release required | 状态 | 差异 |
| --- | --- | --- | --- | --- | --- |
${rows}

正式发布门槛固定为 100.00%、0 blocked、0 in_progress、0 not_started、0 missing evidence。非 required 项只允许用于记录双方明确排除的源码能力。
`;
await Bun.write(new URL('../docs/parity.md', import.meta.url), markdown);
console.log(`Generated docs/parity.md (${summary.percent.toFixed(2)}%)`);
