from typing import Callable, Literal

class CancellationToken:
    """
    取消令牌，用于控制下载任务的生命周期。
    """
    def __init__(self) -> None: ...
    def cancel(self) -> None:
        """触发取消操作，使关联的任务停止。"""
        ...
    def is_cancelled(self) -> bool:
        """返回令牌是否已被取消。"""
        ...

class Config:
    """
    下载任务配置项。所有字段均为可选，未设置时将使用合理的默认值。
    """

    threads: int | None
    """线程数量，默认值 `32`。"""

    proxy: Literal["no", "system"] | str | None
    """
    设置代理，默认值 `system`。支持 https、http、socks5 代理。
    - `"no"`: 不使用代理。
    - `"system"`: 使用系统代理。
    - `proxy_str`: 自定义代理地址，例如 "http://127.0.0.1:7890"。
    """

    headers: dict[str, str] | None
    """自定义 HTTP 请求头。"""

    min_chunk_size: int | None
    """
    最小分块大小，单位为字节，默认值 `500 * 1024` (500KiB)。
    """

    write_buffer_size: int | None
    """
    写入缓冲区大小，单位为字节，默认值 `16 * 1024 * 1024` (16MiB)。
    """

    write_queue_cap: int | None
    """
    写入队列容量，默认值 `10240`。
    """

    retry_gap_ms: int | None
    """
    请求失败后的默认重试间隔，单位毫秒，默认值 `500ms`。
    """

    pull_timeout_ms: int | None
    """
    拉取超时时间，单位毫秒，默认值 `5000ms`。
    """

    accept_invalid_certs: bool | None
    """是否接受无效或自签名证书（危险），默认值 `false`。"""

    accept_invalid_hostnames: bool | None
    """是否接受无效主机名（危险），默认值 `false`。"""

    write_method: Literal["mmap", "std"] | None
    """
    写入磁盘的方式，默认值 `"mmap"`。
    """

    retry_times: int | None
    """设置获取元数据的重试次数，默认值 `3`。"""

    local_address: list[str] | None
    """指定发起请求的本地网卡 IP 地址列表。"""

    max_speculative: int | None
    """
    冗余线程数，默认值 `3`。
    """

    downloaded_chunk: list[tuple[int, int]] | None
    """已经下载过的部分（区间列表）。"""

    chunk_window: int | None
    """
    已下载分块的平滑窗口，单位字节，默认值 `8 * 1024` (8KiB)。
    """

    def __init__(
        self,
        threads: int | None = None,
        proxy: Literal["no", "system"] | str | None = None,
        headers: dict[str, str] | None = None,
        min_chunk_size: int | None = None,
        write_buffer_size: int | None = None,
        write_queue_cap: int | None = None,
        retry_gap_ms: int | None = None,
        pull_timeout_ms: int | None = None,
        accept_invalid_certs: bool | None = None,
        accept_invalid_hostnames: bool | None = None,
        write_method: Literal["mmap", "std"] | None = None,
        retry_times: int | None = None,
        local_address: list[str] | None = None,
        max_speculative: int | None = None,
        downloaded_chunk: list[tuple[int, int]] | None = None,
        chunk_window: int | None = None,
    ) -> None:
        """
        初始化下载配置。

        :param threads: 线程数量，默认值 `32`。线程越多不意味着越快。
        :param proxy: 设置代理，默认值 `system`。支持 https、http、socks5。'no' 表示不使用代理。
        :param headers: 自定义请求头字典。
        :param min_chunk_size: 最小分块大小（字节），默认 `512000` (500KiB)。太小会增加竞争开销。
        :param write_buffer_size: 写入缓冲区大小（字节），默认 `16MiB`。仅对 'std' 写入模式有效。
        :param write_queue_cap: 写入队列容量，默认 `10240`。写满后会触发背压降低下载速度。
        :param retry_gap_ms: 请求失败后的默认重试间隔（毫秒），默认 `500ms`。
        :param pull_timeout_ms: 拉取超时时间（毫秒），默认 `5000ms`。超时将触发重新连接。
        :param accept_invalid_certs: 是否接受无效证书（危险），默认 `False`。
        :param accept_invalid_hostnames: 是否接受无效主机名（危险），默认 `False`。
        :param write_method: 写入磁盘方式。'mmap' 最快但 32 位系统有 4GB 限制，'std' 兼容性最好。
        :param retry_times: 获取元数据的重试次数，默认 `3`。
        :param local_address: 使用哪些本地 IP 地址发送请求（多网卡支持）。
        :param max_speculative: 冗余线程数，默认 `3`。解决收尾阶段 99% 卡顿问题。
        :param downloaded_chunk: 已下载的区间列表，如 [(0, 1024), (2048, 4096)]。
        :param chunk_window: 已下载分块的平滑窗口（字节），默认 `8KiB`。用于合并小的下载空洞。
        """
        ...

class UrlInfo:
    """下载目标的元数据信息。"""

    size: int
    raw_name: str
    supports_range: bool
    fast_download: bool
    final_url: str
    etag: str | None
    last_modified: str | None
    content_type: str | None

    def filename(self) -> str:
        """返回清洗后的安全文件名。"""
        ...

class Event:
    """
    下载过程中的事件对象。
    """

    type: Literal[
        "PrefetchError",
        "Pulling",
        "PullError",
        "PullTimeout",
        "PullProgress",
        "Pushing",
        "PushError",
        "PushProgress",
        "Flushing",
        "FlushError",
        "Finished",
    ]
    id: int | None
    message: str | None
    range: tuple[int, int] | None

class DownloadTask:
    """
    下载任务句柄，用于控制下载流程。
    """
    @property
    def info(self) -> UrlInfo:
        """获取任务的文件元数据信息。"""
        ...

    def cancel(self) -> None:
        """彻底取消下载任务，不可恢复。"""
        ...

    def is_cancelled(self) -> bool:
        """检查任务是否已彻底取消。"""
        ...

    def pause(self) -> None:
        """暂停下载任务，可通过再次调用 start 系列方法恢复。"""
        ...

    def is_paused(self) -> bool:
        """检查任务是否已暂停。"""
        ...

    async def start(
        self, save_path: str, callback: Callable[[Event], None] | None = None
    ) -> None:
        """
        将文件下载并保存到指定路径。

        :param save_path: 磁盘存储路径。
        :param callback: 事件回调函数，用于接收进度和错误。
        """
        ...

    async def start_in_memory(
        self, callback: Callable[[Event], None] | None = None
    ) -> bytes:
        """
        将文件下载到内存中并返回 bytes 数据。

        :param callback: 事件回调函数。
        :return: 下载完成的文件数据。
        """
        ...

    async def start_with_pusher(
        self,
        push_fn: Callable[[int, bytes], None],
        flush_fn: Callable[[], None] | None = None,
        callback: Callable[[Event], None] | None = None,
    ) -> None:
        """
        使用自定义推送器处理下载数据流。

        :param push_fn: 数据处理函数，接收 (offset, data)。
        :param flush_fn: 缓冲区刷新函数。
        :param callback: 事件回调函数。
        """
        ...

async def prefetch(
    url: str, config: Config | None = None, token: CancellationToken | None = None
) -> DownloadTask:
    """
    解析 URL 获取文件信息并准备下载任务。

    :param url: 下载链接。
    :param config: 任务配置项。
    :param token: 可选的取消令牌。
    :return: DownloadTask 实例。
    """
    ...
