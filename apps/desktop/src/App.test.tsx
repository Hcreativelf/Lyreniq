import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { readRuntimeLabel } from "./services/desktop";

vi.mock("./services/desktop", () => ({ readRuntimeLabel: vi.fn() }));
const readRuntime = vi.mocked(readRuntimeLabel);

beforeEach(() => {
  readRuntime.mockReset();
});

describe("workspace shell", () => {
  beforeEach(() => readRuntime.mockResolvedValue("Lyreniq 0.1.0 · windows"));

  it("navigates through all workspaces and moves focus to the page heading", async () => {
    const user = userEvent.setup();
    render(<App />);
    const nav = within(screen.getByRole("navigation", { name: "工作区导航" }));
    const actions = [
      ["文件分析", "导入音频"],
      ["参考音", "播放参考音"],
      ["实时调音", "开始监听"],
    ];
    for (const [label, action] of actions) {
      await user.click(nav.getByRole("button", { name: label }));
      expect(nav.getByRole("button", { name: label })).toHaveAttribute("aria-current", "page");
      expect(screen.getByRole("heading", { name: label, level: 1 })).toHaveFocus();
      expect(screen.getByRole("button", { name: action })).toBeDisabled();
    }
    await user.click(nav.getByRole("button", { name: "设置" }));
    expect(screen.getByRole("heading", { name: "设置", level: 1 })).toHaveFocus();
    expect(screen.getByRole("group", { name: "外观" })).toBeVisible();
  });

  it("keeps the selected theme across navigation and both theme controls agree", async () => {
    const user = userEvent.setup();
    const { container } = render(<App />);
    const shell = container.querySelector(".app-shell");
    expect(shell).toHaveAttribute("data-theme", "dark");
    await user.click(screen.getByRole("button", { name: "切换为浅色主题" }));
    await user.click(screen.getByRole("button", { name: "设置" }));
    expect(shell).toHaveAttribute("data-theme", "light");
    expect(screen.getByRole("button", { name: "浅色主题" })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
    await user.click(screen.getByRole("button", { name: "深色主题" }));
    expect(shell).toHaveAttribute("data-theme", "dark");
    expect(screen.getByRole("button", { name: "切换为浅色主题" })).toBeVisible();
    expect(screen.getByRole("button", { name: "浅色主题" })).toHaveAttribute(
      "aria-pressed",
      "false",
    );
  });

  it("supports keyboard entry and does not show fabricated measurements", async () => {
    const user = userEvent.setup();
    render(<App />);
    await user.tab();
    expect(screen.getByRole("link", { name: "跳到主要内容" })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole("button", { name: "实时调音" })).toHaveFocus();
    await user.tab();
    await user.keyboard("{Enter}");
    expect(screen.getByRole("heading", { name: "文件分析", level: 1 })).toHaveFocus();
    expect(screen.queryByText(/440[.0]*\s*Hz|cents|A4/)).not.toBeInTheDocument();
    expect(readRuntime).toHaveBeenCalledTimes(1);
  });
});

describe("desktop connection", () => {
  it("shows the native result only after the bridge responds", async () => {
    readRuntime.mockResolvedValue("Lyreniq 0.1.0 · windows");
    render(<App />);
    expect(screen.getByRole("button", { name: "重新验证桌面连接" })).toBeDisabled();
    expect(screen.getByText("正在验证桌面连接…")).toBeVisible();
    expect(await screen.findByText("Lyreniq 0.1.0 · windows")).toBeVisible();
    expect(screen.getByRole("button", { name: "重新验证桌面连接" })).toBeEnabled();
  });

  it("reports a failed bridge and can recover when the user retries", async () => {
    readRuntime.mockRejectedValueOnce(new Error("IPC unavailable"));
    readRuntime.mockResolvedValueOnce("Lyreniq 0.1.0 · windows");
    render(<App />);
    expect(await screen.findByText(/无法连接桌面后端/)).toBeVisible();
    expect(screen.queryByText("桌面连接已建立")).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "重新验证桌面连接" }));
    expect(await screen.findByText("Lyreniq 0.1.0 · windows")).toBeVisible();
    expect(readRuntime).toHaveBeenCalledTimes(2);
  });

  it("removes a previous success if a later connection attempt fails", async () => {
    readRuntime.mockResolvedValueOnce("Lyreniq 0.1.0 · windows");
    readRuntime.mockRejectedValueOnce(new Error("Disconnected"));
    render(<App />);
    await screen.findByText("Lyreniq 0.1.0 · windows");
    await userEvent.click(screen.getByRole("button", { name: "重新验证桌面连接" }));
    expect(await screen.findByText(/无法连接桌面后端/)).toBeVisible();
    expect(screen.queryByText("Lyreniq 0.1.0 · windows")).not.toBeInTheDocument();
  });
});
