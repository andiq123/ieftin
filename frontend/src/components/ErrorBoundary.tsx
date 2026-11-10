import { Component, type ErrorInfo, type ReactNode } from 'react'
import { Notice } from './Notice.tsx'

interface Props {
  children: ReactNode
}

interface State {
  failed: boolean
}

/** Keeps a rendering error from leaving a blank page: says what to do instead. */
export class ErrorBoundary extends Component<Props, State> {
  state: State = { failed: false }

  static getDerivedStateFromError(): State {
    return { failed: true }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(error, info.componentStack)
  }

  render() {
    if (!this.state.failed) return this.props.children
    return (
      <main className="page">
        <Notice
          icon="alert"
          title="Couldn't show this page"
          alert
          action={{ label: 'Reload', onClick: () => window.location.reload() }}
        >
          The server may be older than the page. Restart <code>./start.sh</code>, then reload.
        </Notice>
      </main>
    )
  }
}
