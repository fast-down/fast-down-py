import asyncio
import time
from pathlib import Path

import fastdown


async def main():
    url = "https://mirrors.tuna.tsinghua.edu.cn/archlinux/iso/2026.02.01/archlinux-x86_64.iso"
    config = fastdown.Config(
        proxy="no",
        headers={
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Safari/537.36"
        },
    )
    task = await fastdown.prefetch(url, config)
    filename = task.info.filename()
    save_dir = Path("download")
    save_dir.mkdir(parents=True, exist_ok=True)
    path = str(save_dir / filename)
    print(f"Saving to: {path}")

    file_size = task.info.size
    progress_ranges: list[tuple[int, int]] = []
    download_start_time = 0
    flush_start_time = 0

    def print_progress():
        curr_size = sum(r[1] - r[0] for r in progress_ranges)
        percentage = (curr_size / file_size) * 100 if file_size > 0 else 0
        print(f"\rProgress: {curr_size}/{file_size} ({percentage:.2f}%)", end="")

    async def interval_printer():
        while not task.is_cancelled():
            print_progress()
            await asyncio.sleep(1)

    def on_event(event: fastdown.Event):
        nonlocal progress_ranges, download_start_time, flush_start_time
        if event.type == "PushProgress":
            if event.range[0] == 0 and not task.info.fast_download:
                progress_ranges = []
            fastdown.merge_progress(progress_ranges, event.range)
        elif event.type == "Flushing":
            print_progress()
            _ = printer_task.cancel()
            download_end_time = time.perf_counter()
            print(
                f"\nDownload finished in {download_end_time - download_start_time:.2f}s"
            )
            flush_start_time = time.perf_counter()
            print("Flushing to disk...")

    printer_task = asyncio.create_task(interval_printer())
    download_start_time = time.perf_counter()
    await task.start(path, callback=on_event)
    if flush_start_time > 0:
        print(f"Flush finished in {time.perf_counter() - flush_start_time:.2f}s")


if __name__ == "__main__":
    asyncio.run(main())
