using BerryAIGen.Application.DTOs;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Queries.Tags;

public class GetTagByIdQueryHandler : IRequestHandler<GetTagByIdQuery, TagDto?>
{
    private readonly ITagRepository _tagRepository;

    public GetTagByIdQueryHandler(ITagRepository tagRepository) => _tagRepository = tagRepository;

    public async Task<TagDto?> Handle(GetTagByIdQuery request, CancellationToken cancellationToken)
    {
        var tag = await _tagRepository.GetByIdAsync(TagId.From(request.Id), cancellationToken);
        return tag == null ? null : new TagDto
        {
            Id = tag.Id, Name = tag.Name, Color = tag.Color,
            Category = tag.Category, CreatedAt = tag.CreatedAt
        };
    }
}

public class GetTagsQueryHandler : IRequestHandler<GetTagsQuery, List<TagSummaryDto>>
{
    private readonly ITagRepository _tagRepository;

    public GetTagsQueryHandler(ITagRepository tagRepository) => _tagRepository = tagRepository;

    public async Task<List<TagSummaryDto>> Handle(GetTagsQuery request, CancellationToken cancellationToken)
    {
        var tags = request.Category != null
            ? await _tagRepository.GetByCategoryAsync(request.Category, cancellationToken)
            : await _tagRepository.GetAllAsync(cancellationToken);

        return tags.Select(t => new TagSummaryDto
        {
            Id = t.Id, Name = t.Name, Color = t.Color, Category = t.Category
        }).ToList();
    }
}

public class GetTagByNameQueryHandler : IRequestHandler<GetTagByNameQuery, TagDto?>
{
    private readonly ITagRepository _tagRepository;

    public GetTagByNameQueryHandler(ITagRepository tagRepository) => _tagRepository = tagRepository;

    public async Task<TagDto?> Handle(GetTagByNameQuery request, CancellationToken cancellationToken)
    {
        var tag = await _tagRepository.GetByNameAsync(request.Name, cancellationToken);
        return tag == null ? null : new TagDto
        {
            Id = tag.Id, Name = tag.Name, Color = tag.Color,
            Category = tag.Category, CreatedAt = tag.CreatedAt
        };
    }
}