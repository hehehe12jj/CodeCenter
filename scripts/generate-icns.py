#!/usr/bin/env python3
"""
生成 macOS icns 图标文件
使用 PNG 文件创建带圆角的 icns
"""

import os
import struct
import subprocess
from PIL import Image

def create_icns_from_png(png_path, output_path):
    """从 PNG 文件创建 icns 文件"""

    # 使用 macOS 的 iconutil 工具
    temp_dir = output_path.replace('.icns', '.iconset')
    os.makedirs(temp_dir, exist_ok=True)

    img = Image.open(png_path)
    base_name = os.path.basename(png_path).replace('.png', '')

    sizes = [
        (16, 'icon_16x16'),
        (32, 'icon_16x16@2x'),
        (32, 'icon_32x32'),
        (64, 'icon_32x32@2x'),
        (128, 'icon_128x128'),
        (256, 'icon_128x128@2x'),
        (256, 'icon_256x256'),
        (512, 'icon_256x256@2x'),
        (512, 'icon_512x512'),
        (1024, 'icon_512x512@2x'),
    ]

    for size, name in sizes:
        # 调整图像大小
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        out_file = os.path.join(temp_dir, f'{name}.png')
        resized.save(out_file)
        print(f"  创建: {out_file}")

    # 使用 iconutil 转换
    result = subprocess.run(
        ['iconutil', '-c', 'icns', '-o', output_path, temp_dir],
        capture_output=True,
        text=True
    )

    # 清理临时目录
    import shutil
    shutil.rmtree(temp_dir)

    if result.returncode == 0:
        print(f"  icns 文件已创建: {output_path}")
        return True
    else:
        print(f"  iconutil 失败: {result.stderr}")
        return False

def main():
    icons_dir = 'src-tauri/icons'
    png_path = os.path.join(icons_dir, '256x256.png')
    icns_path = os.path.join(icons_dir, 'icon.icns')

    if os.path.exists(png_path):
        print(f"从 {png_path} 创建 icns...")
        create_icns_from_png(png_path, icns_path)
        print("完成!")
    else:
        print(f"找不到 PNG 文件: {png_path}")

if __name__ == "__main__":
    main()
