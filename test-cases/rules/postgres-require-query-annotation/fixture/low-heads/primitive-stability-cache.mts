import { read } from '@bench/db'

declare function replacement(value: unknown): string

function stableGroup() {
  function impossible(value) { throw new Error(String(value)) }
  function producer0(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer1(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer2(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer3(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer4(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer5(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer6(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer7(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function consume0(kind) {
    const q = producer0(kind)
    read(q) // low: true
  }
  function consume1(kind) {
    const q = producer1(kind)
    read(q) // low: true
  }
  function consume2(kind) {
    const q = producer2(kind)
    read(q) // low: true
  }
  function consume3(kind) {
    const q = producer3(kind)
    read(q) // low: true
  }
  function consume4(kind) {
    const q = producer4(kind)
    read(q) // low: true
  }
  function consume5(kind) {
    const q = producer5(kind)
    read(q) // low: true
  }
  function consume6(kind) {
    const q = producer6(kind)
    read(q) // low: true
  }
  function consume7(kind) {
    const q = producer7(kind)
    read(q) // low: true
  }
}

function mutatedGroup() {
  function impossible(value) { throw new Error(String(value)) }
  function producer0(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer1(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer2(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer3(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer4(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer5(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer6(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  function producer7(kind) {
    switch (kind) {
      case 'one': return '/* readOne */ SELECT 1'
      case 'two': return '/* readTwo */ SELECT 2'
      default: return impossible(kind)
    }
  }
  impossible = replacement
  function consume0(kind) {
    const q = producer0(kind)
    read(q) // low: false
  }
  function consume1(kind) {
    const q = producer1(kind)
    read(q) // low: false
  }
  function consume2(kind) {
    const q = producer2(kind)
    read(q) // low: false
  }
  function consume3(kind) {
    const q = producer3(kind)
    read(q) // low: false
  }
  function consume4(kind) {
    const q = producer4(kind)
    read(q) // low: false
  }
  function consume5(kind) {
    const q = producer5(kind)
    read(q) // low: false
  }
  function consume6(kind) {
    const q = producer6(kind)
    read(q) // low: false
  }
  function consume7(kind) {
    const q = producer7(kind)
    read(q) // low: false
  }
}
