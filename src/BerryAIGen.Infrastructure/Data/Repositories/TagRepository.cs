using BerryAIGen.Domain.Entities;
using BerryAIGen.Domain.Interfaces.Repositories;
using BerryAIGen.Domain.ValueObjects;
using Microsoft.EntityFrameworkCore;

namespace BerryAIGen.Infrastructure.Data.Repositories;

/// <summary>
/// Repository implementation for Tag entity using Entity Framework Core.
/// </summary>
public class TagRepository : ITagRepository
{
    private readonly Context.SqliteDbContext _context;

    /// <summary>
    /// Initializes a new instance of the TagRepository class.
    /// </summary>
    public TagRepository(Context.SqliteDbContext context)
    {
        _context = context ?? throw new ArgumentNullException(nameof(context));
    }

    /// <inheritdoc/>
    public async Task<Tag?> GetByIdAsync(TagId id, CancellationToken cancellationToken = default)
    {
        return await _context.Tags
            .FirstOrDefaultAsync(e => e.Id == id, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<Tag?> GetByNameAsync(string name, CancellationToken cancellationToken = default)
    {
        return await _context.Tags
            .FirstOrDefaultAsync(e => e.Name == name, cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Tag>> GetAllAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Tags
            .OrderBy(e => e.Category)
            .ThenBy(e => e.Name)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<List<Tag>> GetByCategoryAsync(string category, CancellationToken cancellationToken = default)
    {
        return await _context.Tags
            .Where(e => e.Category == category)
            .OrderBy(e => e.Name)
            .ToListAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task AddAsync(Tag tag, CancellationToken cancellationToken = default)
    {
        await _context.Tags.AddAsync(tag, cancellationToken);
    }

    /// <inheritdoc/>
    public void Update(Tag tag)
    {
        _context.Tags.Update(tag);
    }

    /// <inheritdoc/>
    public void Delete(Tag tag)
    {
        _context.Tags.Remove(tag);
    }

    /// <inheritdoc/>
    public async Task<int> GetCountAsync(CancellationToken cancellationToken = default)
    {
        return await _context.Tags.CountAsync(cancellationToken);
    }

    /// <inheritdoc/>
    public async Task<bool> ExistsByNameAsync(string name, CancellationToken cancellationToken = default)
    {
        return await _context.Tags
            .AnyAsync(e => e.Name == name, cancellationToken);
    }
}