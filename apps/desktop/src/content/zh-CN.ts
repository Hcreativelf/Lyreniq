export type PageId = "tuner" | "files" | "tone" | "settings";
export type Theme = "dark" | "light";

export const copy = {
  skip: "跳到主要内容",
  sidebar: "Lyreniq 侧栏",
  workspace: "工作区",
  navigation: "工作区导航",
  toLight: "切换为浅色主题",
  toDark: "切换为深色主题",
  lightMode: "浅色模式",
  darkMode: "深色模式",
  local: "为专注而设计 · 本地运行",
  notListening: "未监听",
  unavailable: "尚未开放",
  pending: "待开放",
  appearance: "外观",
  appearanceDescription: "选择适合你的工作环境的界面。",
  currentSession: "本次会话",
  darkTheme: "深色主题",
  lightTheme: "浅色主题",
  audioSettings: "音频与检测",
  audioSettingsDescription: "输入设备、参考频率与检测偏好将在音频功能开放后提供。",
  plannedTools: "待开放工具",
  privacy: "完全本地 · 无账号 · 无音频上传",
  localOnly: "本地运行",
  connection: "桌面连接状态",
  checking: "正在验证桌面连接…",
  connected: "桌面连接已建立",
  connectionError: "无法连接桌面后端",
  reconnect: "重新验证桌面连接",
} as const;

type PageCopy = {
  label: string;
  eyebrow: string;
  description: string;
  action: string;
  panelLabel: string;
  emptyOverline: string;
  emptyTitle: string;
  emptyDescription: string;
  hint: string;
  tools: readonly string[];
};

export const pages: Record<PageId, PageCopy> = {
  tuner: {
    label: "实时调音",
    eyebrow: "TUNING WORKSPACE",
    description: "专注于声音，找到准确的音高。",
    action: "开始监听",
    panelLabel: "调音工作台",
    emptyOverline: "留一点安静，给下一个音",
    emptyTitle: "从一个音，开始。",
    emptyDescription: "音频输入尚未开放，当前不会访问麦克风。",
    hint: "为你的下一次练习做好准备",
    tools: ["音高识别", "实时频谱", "乐器预设"],
  },
  files: {
    label: "文件分析",
    eyebrow: "AUDIO INSIGHT",
    description: "回看每一段声音里的细节。",
    action: "导入音频",
    panelLabel: "音频分析工作台",
    emptyOverline: "听见声音，也看见声音",
    emptyTitle: "给声音一个清晰的轮廓。",
    emptyDescription: "文件分析尚未开放，当前不会读取音频文件。",
    hint: "音频文件将只在本机处理",
    tools: ["本地音频导入", "音高时间轴", "分析结果导出"],
  },
  tone: {
    label: "参考音",
    eyebrow: "REFERENCE TONE",
    description: "让耳朵有一个准确的起点。",
    action: "播放参考音",
    panelLabel: "参考音工作台",
    emptyOverline: "一个稳定的音，一个清晰的起点",
    emptyTitle: "找到你的参照。",
    emptyDescription: "参考音播放尚未开放，当前不会输出声音。",
    hint: "播放开放后，建议从低音量开始",
    tools: ["目标音选择", "音量控制", "本地音频输出"],
  },
  settings: {
    label: "设置",
    eyebrow: "YOUR WORKSPACE",
    description: "把工作区调整成你习惯的样子。",
    action: "",
    panelLabel: "",
    emptyOverline: "",
    emptyTitle: "",
    emptyDescription: "",
    hint: "",
    tools: [],
  },
};
