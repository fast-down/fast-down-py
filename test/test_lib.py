import asyncio
import hashlib
import os
import time
from pathlib import Path

import fastdown
import pytest

# --- 配置与常量 ---
URL = (
    "https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso"
)
EXPECTED_HASH = "c0ee0dab0a181c1d6e3d290a81ae9bc41c329ecaa00816ca7d62a685aeb8d972"
SAVE_DIR = Path("download")
HEADERS = {
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36"
}


# --- 工具函数 ---
def format_size(size: float) -> str:
    for unit in ["B", "KiB", "MiB", "GiB", "TiB"]:
        if size < 1024.0:
            return f"{size:.2f} {unit}"
        size /= 1024.0
    return f"{size:.2f} PiB"


def get_file_sha256(path: str) -> str:
    sha = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(8192 * 16):
            sha.update(chunk)
    return sha.hexdigest()


@pytest.fixture(scope="session", autouse=True)
def setup_download_dir():
    SAVE_DIR.mkdir(parents=True, exist_ok=True)


@pytest.mark.asyncio
async def test_mmap_pause_resume():
    """
    高度严谨的暂停/恢复测试
    对应 JS: test.serial('mmap 写入测试-有中断', ...)
    """
    task = await fastdown.prefetch(
        URL, config=fastdown.Config(proxy="no", headers=HEADERS)
    )
    path = str(SAVE_DIR / f"paused-{task.info.filename()}")
    if os.path.exists(path):
        os.remove(path)

    # 1. 初始状态校验
    assert not task.is_cancelled(), "初始状态不应是已取消"
    assert task.is_paused(), "初始状态应是已暂停"

    # 2. 模拟异步暂停触发器
    async def trigger_pause_later():
        await asyncio.sleep(2.0)  # 运行2秒后操作
        print("\n[Action] Triggering pause...")

        assert not task.is_cancelled()
        assert not task.is_paused()

        task.pause()  # 执行暂停

        assert not task.is_cancelled()
        assert task.is_paused()
        print("[Status] Task is now paused.")

    _pause_task = asyncio.create_task(trigger_pause_later())

    start_time = time.perf_counter()

    # 第一次运行：应该因为暂停而返回
    print("[Run 1] Starting download...")
    await task.start(path)

    # 3. 暂停后的中间状态校验
    print("[Status] First start returned. Verifying states...")
    assert not task.is_cancelled(), "暂停不应导致取消"
    assert task.is_paused(), "任务应该处于暂停状态"

    # 4. 恢复下载
    print("[Run 2] Resuming download...")
    await task.start(path)

    # 5. 校验恢复后的状态
    assert not task.is_cancelled()
    assert task.is_paused()

    # 6. 执行取消并校验最终状态
    task.cancel()
    assert task.is_cancelled(), "执行 cancel 后状态应为已取消"
    assert task.is_paused()

    end_time = time.perf_counter()
    speed = task.info.size / (end_time - start_time)
    print(f"Total time with interruption: {end_time - start_time:.2f}s")
    print(f"Resumed Speed: {format_size(speed)}/s")

    # 7. Hash 校验
    assert get_file_sha256(path) == EXPECTED_HASH
    print("[Success] Hash verified.")


@pytest.mark.asyncio
async def test_custom_pusher_file_io():
    """
    对应 JS: test.serial('自定义写入器测试-Node File API', ...)
    """
    task = await fastdown.prefetch(
        URL, config=fastdown.Config(proxy="no", headers=HEADERS)
    )
    path = str(SAVE_DIR / f"node-api-emu-{task.info.filename()}")
    if os.path.exists(path):
        os.remove(path)

    # 使用底层 fd 模拟 Node 的 fs.open
    f = open(path, "wb")
    try:
        print(f"\n[Run] Downloading with Custom Pusher to {path}...")
        start = time.perf_counter()

        # 定义 Python 版的 pusher
        def py_pusher(offset: int, data: bytes):
            _ = f.seek(offset)
            _ = f.write(data)

        await task.start_with_pusher(push_fn=py_pusher)

        end = time.perf_counter()
        print(f"Speed: {format_size(task.info.size / (end - start))}/s")
    finally:
        f.close()

    assert get_file_sha256(path) == EXPECTED_HASH


@pytest.mark.asyncio
async def test_custom_pusher_memory():
    """
    对应 JS: test.serial('自定义写入器测试-写入内存', ...)
    """
    task = await fastdown.prefetch(
        URL, config=fastdown.Config(proxy="no", headers=HEADERS)
    )

    file_size = task.info.size
    mem_buffer = bytearray(file_size)  # 对应 JS 的 new Uint8Array(fileSize)

    start = time.perf_counter()

    def mem_push(offset: int, data: bytes):
        mem_buffer[offset : offset + len(data)] = data

    await task.start_with_pusher(push_fn=mem_push)

    end = time.perf_counter()
    print(f"\n[Run] Memory Pusher Speed: {format_size(file_size / (end - start))}/s")

    actual_hash = hashlib.sha256(mem_buffer).hexdigest()
    assert actual_hash == EXPECTED_HASH


@pytest.mark.asyncio
async def test_mmap_standard():
    """
    对应 JS: test.serial('mmap 写入测试', ...)
    """
    task = await fastdown.prefetch(
        URL, config=fastdown.Config(proxy="no", headers=HEADERS)
    )
    path = str(SAVE_DIR / f"mmap-std-{task.info.filename()}")
    if os.path.exists(path):
        os.remove(path)

    start = time.perf_counter()

    # 即使不传 callback，也要确保流程正确
    await task.start(path)

    end = time.perf_counter()
    print(
        f"\n[Run] Standard Mmap Speed: {format_size(task.info.size / (end - start))}/s"
    )
    assert get_file_sha256(path) == EXPECTED_HASH


@pytest.mark.asyncio
async def test_start_in_memory_native():
    """
    对应 JS: test.serial('下载到内存测试', ...)
    """
    task = await fastdown.prefetch(
        URL, config=fastdown.Config(proxy="no", headers=HEADERS)
    )

    start = time.perf_counter()
    # 对应 JS 的 const data = await task.startInMemory()
    data = await task.start_in_memory()
    end = time.perf_counter()

    print(
        f"\n[Run] Native In-Memory Speed: {format_size(task.info.size / (end - start))}/s"
    )

    actual_hash = hashlib.sha256(data).hexdigest()
    assert actual_hash == EXPECTED_HASH
