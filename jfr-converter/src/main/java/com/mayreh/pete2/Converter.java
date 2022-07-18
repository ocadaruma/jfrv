package com.mayreh.pete2;

import static java.util.stream.Collectors.toList;

import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import com.fasterxml.jackson.databind.ObjectMapper;

import jdk.jfr.consumer.RecordedEvent;
import jdk.jfr.consumer.RecordedMethod;
import jdk.jfr.consumer.RecordedStackTrace;
import jdk.jfr.consumer.RecordedThread;
import jdk.jfr.consumer.RecordingFile;
import lombok.AllArgsConstructor;
import lombok.Data;

public class Converter {
    @Data
    @AllArgsConstructor
    static class Frame {
        private String typeName;
        private String methodName;
    }

    @Data
    @AllArgsConstructor
    static class Sample {
        private long timestamp;
        private long threadId;
        private int threadStateId;
        private int stackTraceId;
    }

    @Data
    @AllArgsConstructor
    static class StackTrace {
        private List<Frame> frames;
    }

    @Data
    @AllArgsConstructor
    static class Profile {
        private List<Sample> samples;
        private Map<Integer, StackTrace> stackTracePool;
        private Map<Long, String> threadNamePool;
        private Map<Integer, String> threadStatePool;
    }

    public static void main(String[] args) throws IOException {
        Path input = Paths.get(args[0]);
        Path output = Paths.get(args[1]);

        Pool<StackTrace> stackTracePool = new Pool<>();
        Map<Long, String> threadNamePool = new HashMap<>();
        Pool<String> threadStatePool = new Pool<>();

        List<Sample> samples = new ArrayList<>();
        try (RecordingFile jfr = new RecordingFile(input)) {
            while (jfr.hasMoreEvents()) {
                RecordedEvent event = jfr.readEvent();
                if (!"jdk.ExecutionSample".equals(event.getEventType().getName())) {
                    continue;
                }

                RecordedStackTrace recordedTrace = event.getStackTrace();
                RecordedThread thread = event.getValue("sampledThread");
                String threadState = event.getValue("state");

                StackTrace stackTrace = new StackTrace(
                        recordedTrace.getFrames()
                                     .stream()
                                     .map(f -> {
                                         RecordedMethod m = f.getMethod();
                                         return new Frame(m.getType().getName(), m.getName());
                                     })
                                     .collect(toList()));

                final String threadName;
                if (thread.getJavaName() != null) {
                    threadName = thread.getJavaName();
                } else if (thread.getOSName() != null) {
                    threadName = thread.getOSName();
                } else {
                    threadName = "unknown";
                }
                threadNamePool.put(thread.getId(), threadName);
                samples.add(new Sample(event.getStartTime().toEpochMilli(),
                                       thread.getId(),
                                       threadStatePool.register(threadState),
                                       stackTracePool.register(stackTrace)));
            }

            Profile profile = new Profile(samples,
                                          stackTracePool.result(),
                                          threadNamePool,
                                          threadStatePool.result());

            new ObjectMapper().writeValue(output.toFile(), profile);
        }
    }

    private static class Pool<T> {
        private int id = 0;
        private final Map<T, Integer> cache = new HashMap<>();

        public Map<Integer, T> result() {
            Map<Integer, T> map = new HashMap<>();
            cache.forEach((k, v) -> map.put(v, k));
            return map;
        }

        public int register(T value) {
            Integer i = cache.get(value);
            if (i != null) {
                return i;
            }

            cache.put(value, id);
            return id++;
        }
    }
}
