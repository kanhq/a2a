// written by LLM provider: vertex-ai model: gemini-2.5-flash-preview-05-20
async function main(config, params) {
  // 定义输入和输出目录
  const inputDir = 'images';
  const outputDir = 'images_out';

  // 1. 检查并创建输出目录
  // 使用 mkdir -p 确保目录存在，如果不存在则创建
  await doAction({
    kind: "shell",
    command: "mkdir",
    args: ["-p", outputDir]
  });

  // 2. 列出 images 目录中的所有图片文件
  // 使用 find 命令查找指定目录下，文件类型为f（普通文件），且文件名匹配常见图片扩展名的文件
  const imageListResult = await doAction({
    kind: "shell",
    command: "find",
    args: [inputDir, "-type", "f", "-regex", ".*\\.\\(jpg\\|jpeg\\|png\\|gif\\|bmp\\)"]
  });

  // 将结果按行分割，得到图片文件路径数组
  const imagePaths = imageListResult.split('\n').filter(path => path.trim() !== '');

  // 3. 遍历每个图片文件并进行处理
  let lastResult = null; // 用于存储最后一个操作的结果

  for (const imagePath of imagePaths) {
    // 从完整路径中提取文件名，不包含目录
    const fileNameWithExt = imagePath.split('/').pop();
    // 构造输出文件名，保持原文件名，但扩展名为 .jpg
    const outputFileName = fileNameWithExt.split('.').slice(0, -1).join('.') + '.jpg';
    // 构造完整的输出文件路径
    const outputPath = `${outputDir}/${outputFileName}`;

    // 使用 ImageMagick (magick 命令) 进行图片处理
    // -resize "1024x>" 表示如果图片宽度大于1024，则按比例缩小，最大宽度为1024。
    // 保持宽高比，并转换为 JPG 格式。
    lastResult = await doAction({
      kind: "shell",
      command: "magick",
      args: [imagePath, "-resize", "1024x>", outputPath]
    });
  }

  // 返回最后一个图片处理操作的结果
  // 如果没有图片，则返回创建目录的结果
  return lastResult || `Directory '${outputDir}' created or already exists.`;
}
