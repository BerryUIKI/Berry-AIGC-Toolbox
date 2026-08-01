using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using MediatR;

namespace BerryAIGen.Application.Commands.Tags;

public class CreateTagCommandHandler : IRequestHandler<CreateTagCommand, Guid>
{
    private readonly ITagRepository _tagRepository;
    private readonly IUnitOfWork _unitOfWork;

    public CreateTagCommandHandler(ITagRepository tagRepository, IUnitOfWork unitOfWork)
    {
        _tagRepository = tagRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Guid> Handle(CreateTagCommand request, CancellationToken cancellationToken)
    {
        var tag = Domain.Entities.Tag.Create(request.Name, request.Color, request.Category);
        await _tagRepository.AddAsync(tag, cancellationToken);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return tag.Id;
    }
}

public class UpdateTagCommandHandler : IRequestHandler<UpdateTagCommand, Unit>
{
    private readonly ITagRepository _tagRepository;
    private readonly IUnitOfWork _unitOfWork;

    public UpdateTagCommandHandler(ITagRepository tagRepository, IUnitOfWork unitOfWork)
    {
        _tagRepository = tagRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(UpdateTagCommand request, CancellationToken cancellationToken)
    {
        var tag = await _tagRepository.GetByIdAsync(TagId.From(request.Id), cancellationToken)
            ?? throw new InvalidOperationException($"Tag {request.Id} not found");

        tag.Rename(request.Name);
        if (request.Color != null) tag.SetColor(request.Color);
        if (request.Category != null) tag.SetCategory(request.Category);

        _tagRepository.Update(tag);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}

public class DeleteTagCommandHandler : IRequestHandler<DeleteTagCommand, Unit>
{
    private readonly ITagRepository _tagRepository;
    private readonly IUnitOfWork _unitOfWork;

    public DeleteTagCommandHandler(ITagRepository tagRepository, IUnitOfWork unitOfWork)
    {
        _tagRepository = tagRepository;
        _unitOfWork = unitOfWork;
    }

    public async Task<Unit> Handle(DeleteTagCommand request, CancellationToken cancellationToken)
    {
        var tag = await _tagRepository.GetByIdAsync(TagId.From(request.Id), cancellationToken)
            ?? throw new InvalidOperationException($"Tag {request.Id} not found");

        _tagRepository.Delete(tag);
        await _unitOfWork.SaveChangesAsync(cancellationToken);
        return Unit.Value;
    }
}