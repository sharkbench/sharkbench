FROM mcr.microsoft.com/dotnet/sdk:6.0 AS build

WORKDIR /app

# Dependencies
COPY *.csproj ./
RUN dotnet restore

COPY . ./

RUN dotnet publish -c Release -o out

FROM mcr.microsoft.com/dotnet/aspnet:6.0 AS deploy

WORKDIR /app
COPY --from=build /app/out .

EXPOSE 3000
ENTRYPOINT ["dotnet", "web.dll"]
