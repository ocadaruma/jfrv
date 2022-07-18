export interface Frame {
  typeName: string
  methodName: string
}

export interface Sample {
  // Timestamp in unix epoch millis
  timestamp: number
  // Thread id
  threadId: number
  // State of the thread when the sample is recorded
  threadStateId: number
  // Stack trace of the sample
  stackTraceId: number
}

export interface StackTrace {
  frames: Frame[]
}

export interface Profile {
  samples: Sample[]
  stackTracePool: {[id: number]: StackTrace}
  threadNamePool: {[id: number]: string}
  threadStatePool: {[id: number]: string}
}
