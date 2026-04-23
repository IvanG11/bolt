import { ref, nextTick } from 'vue'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import '@xterm/xterm/css/xterm.css'

export function useShell() {
  const shellService = ref(null)
  const termEl       = ref(null)
  let term     = null
  let termWs   = null
  let fitAddon = null

  async function openShell(project, service) {
    closeShell()
    shellService.value = service
    await nextTick()

    term = new Terminal({
      theme: { background: '#0b0e16', foreground: '#b8c5d8', cursor: '#6366f1' },
      fontFamily: 'IBM Plex Mono, monospace',
      fontSize: 13,
      cursorBlink: true,
    })
    fitAddon = new FitAddon()
    term.loadAddon(fitAddon)
    term.open(termEl.value)
    fitAddon.fit()
    term.focus()

    const proto = location.protocol === 'https:' ? 'wss' : 'ws'
    const url   = `${proto}://${location.host}/api/projects/${project}/shell?service=${encodeURIComponent(service)}`
    termWs = new WebSocket(url)
    termWs.binaryType = 'arraybuffer'

    termWs.onmessage = (e) => {
      if (e.data instanceof ArrayBuffer) term.write(new Uint8Array(e.data))
      else term.write(e.data)
    }
    termWs.onclose = () => term?.write('\r\n\x1b[31m[disconnected]\x1b[0m\r\n')

    term.onData((data) => {
      if (termWs?.readyState === WebSocket.OPEN) termWs.send(data)
    })
    term.onResize(({ cols, rows }) => {
      if (termWs?.readyState === WebSocket.OPEN)
        termWs.send(JSON.stringify({ type: 'resize', cols, rows }))
    })
  }

  function closeShell() {
    termWs?.close()
    termWs = null
    term?.dispose()
    term = null
    shellService.value = null
  }

  return { shellService, termEl, openShell, closeShell }
}
