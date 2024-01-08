<?php

namespace App\Controller;

use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\JsonResponse;
use Symfony\Component\Routing\Annotation\Route;
use Symfony\Component\HttpFoundation\Request;
use Symfony\Contracts\HttpClient\HttpClientInterface;

class AppController extends AbstractController
{
    private $client;

    public function __construct(HttpClientInterface $client)
    {
        $this->client = $client;
    }

    #[Route('/api/v1/periodic-table/element')]
    public function getElement(Request $request): JsonResponse
    {
        $symbol = $request->query->get('symbol');
        $response = $this->client->request('GET', 'http://web-data-source/element.json');
        $json_data = $response->toArray();
        $entry = $json_data[$symbol] ?? null;

        return $this->json([
            'name' => $entry['name'],
            'number' => $entry['number'],
            'group' => $entry['group'],
        ]);
    }

    #[Route('/api/v1/periodic-table/shells')]
    public function getShells(Request $request): JsonResponse
    {
        $symbol = $request->query->get('symbol');
        $response = $this->client->request('GET', 'http://web-data-source/shells.json');
        $json_data = $response->toArray();

        return $this->json([
            'shells' => $json_data[$symbol],
        ]);
    }
}
