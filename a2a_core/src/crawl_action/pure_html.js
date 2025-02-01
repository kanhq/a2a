function cleanup(parent, format) {
  const elementsToRemove = [
    'script',
    'style',
    'noscript',
    'iframe',
    'svg',
    'img',
    'audio',
    'video',
    'canvas',
    'map',
    'source',
    'dialog',
    'menu',
    'menuitem',
    'track',
    'object',
    'embed',
    'form',
    'input',
    // 'button',
    // 'select',
    'textarea',
    // 'label',
    // 'option',
    // 'optgroup',
    'aside',
    'footer',
    'header',
    'nav',
    'head',
  ]

  const attributesToRemove = [
    'style',
    'src',
    'alt',
    'title',
    'role',
    'aria-',
    'tabindex',
    'on',
    'data-',
    'class',
    'target'
  ]

  if (!parent) {
    parent = 'body'
  }

  const parentNodes = document.querySelectorAll(parent);

  if (parentNodes.length === 0) {
    return ''
  }

  const rows = []

  for (let i = 0; i < parentNodes.length; i++) {
    let parent = parentNodes[i]
    if (!parent) {
      continue
    }

    parent = parent.cloneNode(true)

    const elementTree = parent.querySelectorAll('*')

    elementTree.forEach((element) => {

      if (element.innerText.trim() === '') {
        element.remove()
        return
      }

      if (elementsToRemove.includes(element.tagName.toLowerCase())) {
        element.remove()
        return
      }

      Array.from(element.attributes).forEach((attr) => {
        if (attributesToRemove.some((a) => attr.name.startsWith(a))) {
          element.removeAttribute(attr.name)
        }
      })
    })
    if (format === 'text' && parent.textContent) {
      rows.push(parent.textContent)
    } else {
      rows.push(parent.innerHTML)
    }
  }

  return rows.join('\n')
}