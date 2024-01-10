import {Controller, Get, Query} from '@nestjs/common';
import * as http from "http";

// fetch() is not available prior Node.js 18, and it is slower than http.get()
// TODO: Use fetch() as soon as the following issue is resolved:
// https://github.com/nodejs/undici/issues/1203
function fetch(url: string) {
  return new Promise((resolve, reject) => {
    http.get(url, (res) => {
      let data = '';

      res.on('data', (chunk) => {
        data += chunk;
      });

      res.on('end', () => {
        resolve(JSON.parse(data));
      });
    }).on('error', (err) => {
      reject(err);
    });
  });
}

@Controller('api/v1/periodic-table')
export class AppController {
  @Get('element')
  async getElement(@Query('symbol') symbol: string) {
    const elementRes = await fetch('http://web-data-source/element.json');
    const element = elementRes[symbol];

    return {
      name: element.name,
      number: element.number,
      group: element.group
    };
  }

  @Get('shells')
  async getShells(@Query('symbol') symbol: string) {
    const shellsRes = await fetch('http://web-data-source/shells.json');

    return {
      shells: shellsRes[symbol]
    };
  }
}
