import crontrue from 'cronstrue'

type ShortcutOptions = {
  shift?: boolean
  alt?: boolean
  cmdOrCtrl?: boolean
}
const isMac = navigator.userAgent.indexOf('Mac') != -1

export function checkModifiers(e: KeyboardEvent | MouseEvent, options: ShortcutOptions) {
  const target = {
    shift: options.shift || false,
    alt: options.alt || false,
    ctrl: (!isMac && options.cmdOrCtrl) || false,
    meta: (isMac && options.cmdOrCtrl) || false,
  }

  const pressed = {
    shift: !!e.shiftKey,
    alt: !!e.altKey,
    ctrl: !!e.ctrlKey,
    meta: !!e.metaKey,
  }

  return (
    pressed.shift === target.shift &&
    pressed.alt === target.alt &&
    pressed.ctrl === target.ctrl &&
    pressed.meta === target.meta
  )
}

export function checkShortcut(e: KeyboardEvent, key: string, options: ShortcutOptions = {}) {
  if (e.key.toUpperCase() !== key.toUpperCase()) return false
  return checkModifiers(e, options)
}

/**
 * In Safari input elements, when you type something, delete it, tab and then shift+tab, the cursor is invisible. This fixes that
 */
export function invisibleCursorFix(node: HTMLInputElement) {
  const handleFocus = (e: FocusEvent) => {
    if (e.target instanceof HTMLInputElement) {
      // eslint-disable-next-line no-self-assign
      const value = e.target.value
      e.target.value = 'x'
      e.target.value = value
    }
  }
  node.addEventListener('focus', handleFocus)
  return {
    destroy() {
      node.removeEventListener('focus', handleFocus)
    },
  }
}

export function getCronText(cron: string) {
  try {
    return crontrue.toString(cron)
  } catch (e) {
    return 'Invalid'
  }
}
