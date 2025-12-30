<?php

declare(strict_types=1);

namespace App\Presenters;

use Nette\Application\UI\Presenter;
use Nette\Application\BadRequestException;
use GuzzleHttp\Client;

final class ApiPresenter extends Presenter
{
    private Client $httpClient;

    public function __construct(Client $httpClient)
    {
        parent::__construct();
        $this->httpClient = $httpClient;
    }

    /**
     * Disable template rendering
     */
    public function formatLayoutTemplateFiles(): array
    {
        return [];
    }

    public function formatTemplateFiles(): array
    {
        return [];
    }

    /**
     * GET /api/v1/periodic-table/element?symbol={symbol}
     */
    public function actionElement(): void
    {
        $symbol = $this->getParameter('symbol');

        if (!$symbol) {
            throw new BadRequestException('Missing symbol parameter');
        }

        $response = $this->httpClient->get('/element.json');
        $data = json_decode($response->getBody()->getContents(), true);

        $element = $data[$symbol] ?? null;

        if (!$element) {
            throw new BadRequestException('Element not found');
        }

        $this->sendJson([
            'name' => $element['name'],
            'number' => $element['number'],
            'group' => $element['group'],
        ]);
    }

    /**
     * GET /api/v1/periodic-table/shells?symbol={symbol}
     */
    public function actionShells(): void
    {
        $symbol = $this->getParameter('symbol');

        if (!$symbol) {
            throw new BadRequestException('Missing symbol parameter');
        }

        $response = $this->httpClient->get('/shells.json');
        $data = json_decode($response->getBody()->getContents(), true);

        $shells = $data[$symbol] ?? null;

        if ($shells === null) {
            throw new BadRequestException('Shells not found');
        }

        $this->sendJson([
            'shells' => $shells,
        ]);
    }
}
