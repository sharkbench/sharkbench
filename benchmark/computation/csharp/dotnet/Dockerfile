FROM mcr.microsoft.com/dotnet/sdk:6.0 AS build

WORKDIR /app

COPY . ./

RUN dotnet publish -c Release -o out /p:UseAppHost=false

FROM mcr.microsoft.com/dotnet/runtime:6.0 AS deploy

WORKDIR /app
COPY --from=build /app/out .

EXPOSE 3000
ENTRYPOINT ["dotnet", "web.dll"]
