FROM mcr.microsoft.com/dotnet/sdk:10.0 AS build
RUN apt-get update && apt-get install -y --no-install-recommends clang zlib1g-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY Bench.csproj .
RUN dotnet restore
COPY Program.cs .
RUN dotnet publish -c Release -o /app --no-restore

FROM mcr.microsoft.com/dotnet/runtime-deps:10.0-noble-chiseled
WORKDIR /app
COPY --from=build /app/Bench .
ENV DOTNET_PROCESSOR_COUNT=1 \
    DOTNET_gcConcurrent=0 \
    DOTNET_ThreadPool_ForceMaxWorkerThreads=2 \
    DOTNET_GCHeapHardLimit=0x800000 \
    DOTNET_GCConserveMemory=9
EXPOSE 5001
ENTRYPOINT ["./Bench"]
