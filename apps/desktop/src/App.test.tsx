import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { readRuntimeLabel } from "./services/desktop";

vi.mock("./services/desktop", () => ({ readRuntimeLabel: vi.fn() }));
const readRuntime = vi.mocked(readRuntimeLabel);

beforeEach(() => {
  readRuntime.mockReset();
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
