using MediatR;

namespace BerryAIGen.Application.Queries.Tags;

public record GetTagByIdQuery : IRequest<Application.DTOs.TagDto?>
{
    public Guid Id { get; init; }
}

public record GetTagsQuery : IRequest<List<Application.DTOs.TagSummaryDto>>
{
    public string? Category { get; init; }
}

public record GetTagByNameQuery : IRequest<Application.DTOs.TagDto?>
{
    public string Name { get; init; } = string.Empty;
}