#!/usr/bin/env python3
"""
生成 CodeCenter 应用的图标（带圆角）
圆角半径为图标尺寸的 1/4，确保在 Dock 栏显示正确的圆角效果
"""

from PIL import Image, ImageDraw
import os

# 配置
SOURCE_LOGO = "trans_bg.png"
OUTPUT_DIR = "src-tauri/icons"
SIZES = [32, 128, 256]

def create_icon(size, logo_path):
    """创建正方形图标"""
    logo = Image.open(logo_path).convert('RGBA')
    logo = logo.resize((size, size), Image.Resampling.LANCZOS)
    return logo


def create_rounded_icon(size, logo_path, corner_radius=None):
    """创建带圆角的图标

    Args:
        size: 图标尺寸（正方形边长）
        logo_path: 源logo文件路径
        corner_radius: 圆角半径，默认值为尺寸的1/4
    """
    if corner_radius is None:
        corner_radius = size // 4

    # 打开并调整logo大小
    logo = Image.open(logo_path).convert('RGBA')
    logo = logo.resize((size, size), Image.Resampling.LANCZOS)

    # 创建圆角蒙版
    mask = Image.new('L', (size, size), 0)
    draw = ImageDraw.Draw(mask)

    # 绘制圆角矩形
    draw.rounded_rectangle(
        (0, 0, size, size),
        radius=corner_radius,
        fill=255
    )

    # 应用蒙版
    rounded_logo = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    rounded_logo.paste(logo, (0, 0), mask=mask)

    return rounded_logo


def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)

    logo_path = SOURCE_LOGO
    if not os.path.isabs(logo_path):
        logo_path = os.path.join(os.path.dirname(__file__), '..', logo_path)
    logo_path = os.path.abspath(logo_path)

    print(f"源 logo: {logo_path}")

    for size in SIZES:
        print(f"生成 {size}x{size} 圆角图标...")
        icon = create_rounded_icon(size, logo_path)

        png_path = os.path.join(OUTPUT_DIR, f"{size}x{size}.png")
        icon.save(png_path)
        print(f"  已保存: {png_path}")

    # 复制 256x256 作为 @2x 版本
    at2x_path = os.path.join(OUTPUT_DIR, "128x128@2x.png")
    png_256_path = os.path.join(OUTPUT_DIR, "256x256.png")
    if os.path.exists(png_256_path):
        import shutil
        shutil.copy(png_256_path, at2x_path)
        print(f"已复制 256x256 作为 128x128@2x.png")

    print("\n圆角图标生成完成！")


if __name__ == "__main__":
    main()
