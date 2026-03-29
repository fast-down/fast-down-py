from .fastdown import *  # noqa: F403


def merge_progress(arr: list[tuple[int, int]], new_range: tuple[int, int]):
    c_start, c_end = new_range
    left = 0
    right = len(arr)
    while left < right:
        mid = (left + right) // 2
        if arr[mid][1] < c_start:
            left = mid + 1
        else:
            right = mid
    i = left
    if i == len(arr):
        arr.append((c_start, c_end))
        return
    if arr[i][0] <= c_start and arr[i][1] >= c_end:
        return
    j = i
    while j < len(arr):
        entry = arr[j]
        if entry[0] > c_end:
            break
        if entry[0] < c_start:
            c_start = entry[0]
        if entry[1] > c_end:
            c_end = entry[1]
        j += 1
    delete_count = j - i
    if delete_count == 1:
        arr[i] = (c_start, c_end)
    else:
        arr[i:j] = [(c_start, c_end)]
