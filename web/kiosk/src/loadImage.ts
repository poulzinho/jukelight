/**
 * Resolves once the image at `url` has loaded and been decoded, so putting
 * it on screen doesn't show a blank frame first.
 */
export function loadImage(url: string): Promise<void> {
  const image = new Image()
  image.src = url
  return image.decode()
}
